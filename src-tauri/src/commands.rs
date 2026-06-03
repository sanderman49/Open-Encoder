use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_shell::process::{CommandChild, CommandEvent};
use tauri_plugin_shell::ShellExt;

use crate::JobStore;

// ─── Request types ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoProbeResult {
    pub duration: f64,
    pub has_audio: bool,
    pub has_video: bool,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
    pub is_interlaced: bool,
    pub field_order: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeinterlaceConfig {
    pub enabled: bool,
    pub auto_detect: bool,
    pub algorithm: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoConfig {
    pub codec: String,
    pub container: String,
    pub resolution: String,
    pub custom_width: Option<u32>,
    pub custom_height: Option<u32>,
    pub crf: u8,
    pub encode_preset: Option<String>,
    pub deinterlace: DeinterlaceConfig,
    pub hw_accel: String,
    pub vaapi_device: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioExportConfig {
    pub format: String,
    pub bitrate: Option<String>,
    pub sample_rate: u32,
    pub bit_depth: Option<u8>,
    pub channels: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    pub video_dir: String,
    pub audio_dir: String,
    pub audio_dir_relative: bool,
    pub create_date_folder: bool,
    pub filename_prefix: String,
    pub filename_suffix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessRequest {
    pub input_path: String,
    pub output_config: OutputConfig,
    pub video: VideoConfig,
    pub audio_export: Option<AudioExportConfig>,
    pub job_id: String,
    pub probe: VideoProbeResult,
    pub title: String,
}

// ─── Event payloads ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
struct ProgressEvent {
    job_id: String,
    phase: String,
    percent: f64,
    speed: String,
    eta_seconds: f64,
}

#[derive(Debug, Clone, Serialize)]
struct CompleteEvent {
    job_id: String,
    video_output: String,
    audio_output: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ErrorEvent {
    job_id: String,
    error: String,
}

// ─── probe_video ──────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn probe_video(app: AppHandle, input_path: String) -> Result<VideoProbeResult, String> {
    let (mut rx, _child) = app
        .shell()
        .sidecar("binaries/ffprobe")
        .map_err(|e| e.to_string())?
        .args([
            "-v",
            "quiet",
            "-print_format",
            "json",
            "-show_streams",
            "-show_format",
            &input_path,
        ])
        .spawn()
        .map_err(|e| e.to_string())?;

    let mut stdout = String::new();
    while let Some(event) = rx.recv().await {
        match event {
            CommandEvent::Stdout(b) => stdout.push_str(&String::from_utf8_lossy(&b)),
            CommandEvent::Terminated(_) => break,
            _ => {}
        }
    }

    parse_ffprobe_json(&stdout)
}

fn parse_ffprobe_json(json: &str) -> Result<VideoProbeResult, String> {
    let v: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("ffprobe parse error: {e}"))?;

    let duration = v["format"]["duration"]
        .as_str()
        .and_then(|s| s.parse::<f64>().ok())
        .or_else(|| {
            v["streams"].as_array()?.iter()
                .find_map(|s| s["duration"].as_str()?.parse::<f64>().ok())
        })
        .unwrap_or(0.0);

    let mut result = VideoProbeResult {
        duration,
        has_audio: false,
        has_video: false,
        width: None,
        height: None,
        video_codec: None,
        audio_codec: None,
        is_interlaced: false,
        field_order: None,
    };

    if let Some(streams) = v["streams"].as_array() {
        for stream in streams {
            match stream["codec_type"].as_str() {
                Some("video") if !result.has_video => {
                    result.has_video = true;
                    result.width = stream["width"].as_u64().map(|v| v as u32);
                    result.height = stream["height"].as_u64().map(|v| v as u32);
                    result.video_codec = stream["codec_name"].as_str().map(String::from);
                    let fo = stream["field_order"].as_str().unwrap_or("progressive");
                    result.is_interlaced = !matches!(fo, "progressive" | "unknown" | "");
                    result.field_order = Some(fo.to_string());
                }
                Some("audio") if !result.has_audio => {
                    result.has_audio = true;
                    result.audio_codec = stream["codec_name"].as_str().map(String::from);
                }
                _ => {}
            }
        }
    }

    Ok(result)
}

// ─── start_process ────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn start_process(
    app: AppHandle,
    request: ProcessRequest,
    job_store: State<'_, JobStore>,
) -> Result<(), String> {
    let store = job_store.inner().clone();
    let app_clone = app.clone();
    tauri::async_runtime::spawn(async move {
        let job_id = request.job_id.clone();
        if let Err(e) = run_process(app_clone.clone(), request, store).await {
            let _ = app_clone.emit("job-error", ErrorEvent { job_id, error: e });
        }
    });
    Ok(())
}

fn resolve_output_dir(
    configured: &str,
    base_dir: &std::path::Path,
    is_relative: bool,
    create_date_folder: bool,
) -> Result<std::path::PathBuf, String> {
    let base = if configured.is_empty() {
        base_dir.to_path_buf()
    } else if is_relative {
        base_dir.join(configured)
    } else {
        std::path::PathBuf::from(configured)
    };

    let dir = if create_date_folder {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let (y, m, d) = secs_to_ymd(ts);
        base.join(format!("{:04}-{:02}-{:02}", y, m, d))
    } else {
        base
    };

    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create output dir: {e}"))?;
    Ok(dir)
}

fn secs_to_ymd(secs: u64) -> (u32, u32, u32) {
    let days = secs / 86400;
    let z = days + 719468;
    let era = z / 146097;
    let doe = z % 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as u32, m as u32, d as u32)
}

async fn run_process(
    app: AppHandle,
    req: ProcessRequest,
    job_store: Arc<Mutex<HashMap<String, CommandChild>>>,
) -> Result<(), String> {
    let file_stem = Path::new(&req.input_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output")
        .to_string();
    // Title drives both the output filename and the embedded metadata title
    let stem = if req.title.is_empty() { file_stem } else { req.title.clone() };

    let oc = &req.output_config;
    let prefix = &oc.filename_prefix;
    let suffix = &oc.filename_suffix;

    let input_parent = Path::new(&req.input_path)
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from("."));

    let video_dir = resolve_output_dir(&oc.video_dir, &input_parent, false, oc.create_date_folder)?;

    // Audio relative paths are resolved relative to the video output dir, not the source file
    let audio_dir = if oc.audio_dir.is_empty() {
        video_dir.clone()
    } else {
        let audio_base = if oc.audio_dir_relative { video_dir.as_path() } else { input_parent.as_path() };
        resolve_output_dir(&oc.audio_dir, audio_base, oc.audio_dir_relative, false)?
    };

    // ── Video pass ───────────────────────────────────────────────────────────
    let ext = if req.video.codec == "copy" {
        Path::new(&req.input_path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("mp4")
            .to_string()
    } else {
        req.video.container.clone()
    };

    let video_out = video_dir
        .join(format!("{}{}{}.{}", prefix, stem, suffix, ext))
        .to_string_lossy()
        .to_string();
    let video_args = build_video_args(&req, &video_out);

    let (mut rx, child) = app
        .shell()
        .sidecar("binaries/ffmpeg")
        .map_err(|e| e.to_string())?
        .args(&video_args)
        .spawn()
        .map_err(|e| e.to_string())?;

    job_store.lock().unwrap().insert(req.job_id.clone(), child);

    // Stream video progress — parse both stdout and stderr so we catch progress
    // regardless of which pipe FFmpeg actually uses.
    {
        let mut line_buf = String::new();
        let mut block: HashMap<String, String> = HashMap::new();
        let mut stderr_buf = String::new();
        loop {
            let (bytes, is_stderr, done, exit_ok) = match rx.recv().await {
                Some(CommandEvent::Stdout(b)) => (Some(b), false, false, true),
                Some(CommandEvent::Stderr(b)) => (Some(b), true, false, true),
                Some(CommandEvent::Terminated(p)) => (None, false, true, p.code == Some(0)),
                None => (None, false, true, true),
                _ => (None, false, false, true),
            };
            if let Some(b) = bytes {
                let text = String::from_utf8_lossy(&b).to_string();
                if is_stderr { stderr_buf.push_str(&text); }
                line_buf.push_str(&text);
                while let Some(nl) = line_buf.find('\n') {
                    let line = line_buf[..nl].trim().to_string();
                    line_buf.drain(..=nl);
                    if line.is_empty() { continue; }
                    if let Some((k, v)) = line.split_once('=') {
                        block.insert(k.to_string(), v.trim().to_string());
                    }
                    if line.starts_with("progress=") {
                        emit_progress(&app, &block, req.probe.duration, &req.job_id, "video");
                        block.clear();
                    }
                }
            }
            if done {
                let was_cancelled = job_store.lock().unwrap().remove(&req.job_id).is_none();
                if was_cancelled {
                    return Ok(());
                }
                if !exit_ok {
                    let tail: String = stderr_buf.lines().rev().take(5)
                        .collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>().join("\n");
                    return Err(if tail.is_empty() {
                        "FFmpeg (video) exited with non-zero code".into()
                    } else {
                        format!("FFmpeg (video) error:\n{}", tail)
                    });
                }
                break;
            }
        }
    }

    // ── Audio export pass ────────────────────────────────────────────────────
    let audio_out = if let Some(ref cfg) = req.audio_export {
        let out = audio_dir
            .join(format!("{}{}{}.{}", prefix, stem, suffix, cfg.format))
            .to_string_lossy()
            .to_string();
        let args = build_audio_args(&req.input_path, cfg, &out);
        let audio_key = format!("{}-audio", req.job_id);

        let (mut rx2, child2) = app
            .shell()
            .sidecar("binaries/ffmpeg")
            .map_err(|e| e.to_string())?
            .args(&args)
            .spawn()
            .map_err(|e| e.to_string())?;

        job_store.lock().unwrap().insert(audio_key.clone(), child2);

        let mut line_buf2 = String::new();
        let mut block2: HashMap<String, String> = HashMap::new();
        let mut stderr_buf2 = String::new();
        loop {
            let (bytes, is_stderr, done, exit_ok) = match rx2.recv().await {
                Some(CommandEvent::Stdout(b)) => (Some(b), false, false, true),
                Some(CommandEvent::Stderr(b)) => (Some(b), true, false, true),
                Some(CommandEvent::Terminated(p)) => (None, false, true, p.code == Some(0)),
                None => (None, false, true, true),
                _ => (None, false, false, true),
            };
            if let Some(b) = bytes {
                let text = String::from_utf8_lossy(&b).to_string();
                if is_stderr { stderr_buf2.push_str(&text); }
                line_buf2.push_str(&text);
                while let Some(nl) = line_buf2.find('\n') {
                    let line = line_buf2[..nl].trim().to_string();
                    line_buf2.drain(..=nl);
                    if line.is_empty() { continue; }
                    if let Some((k, v)) = line.split_once('=') {
                        block2.insert(k.to_string(), v.trim().to_string());
                    }
                    if line.starts_with("progress=") {
                        emit_progress(&app, &block2, req.probe.duration, &req.job_id, "audio");
                        block2.clear();
                    }
                }
            }
            if done {
                let was_cancelled = job_store.lock().unwrap().remove(&audio_key).is_none();
                if was_cancelled {
                    return Ok(());
                }
                if !exit_ok {
                    let tail: String = stderr_buf2.lines().rev().take(5)
                        .collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>().join("\n");
                    return Err(if tail.is_empty() {
                        "FFmpeg (audio) exited with non-zero code".into()
                    } else {
                        format!("FFmpeg (audio) error:\n{}", tail)
                    });
                }
                break;
            }
        }

        Some(out)
    } else {
        None
    };

    let _ = app.emit(
        "job-complete",
        CompleteEvent {
            job_id: req.job_id.clone(),
            video_output: video_out,
            audio_output: audio_out,
        },
    );

    Ok(())
}

fn parse_out_time(s: &str) -> Option<f64> {
    let s = s.trim();
    if s.starts_with("N/A") { return None; }
    let parts: Vec<&str> = s.splitn(3, ':').collect();
    if parts.len() != 3 { return None; }
    let h: f64 = parts[0].parse().ok()?;
    let m: f64 = parts[1].parse().ok()?;
    let sec: f64 = parts[2].parse().ok()?;
    Some(h * 3600.0 + m * 60.0 + sec)
}

fn emit_progress(
    app: &AppHandle,
    block: &HashMap<String, String>,
    duration: f64,
    job_id: &str,
    phase: &str,
) {
    // out_time_ms is in microseconds despite the name; fall back to out_time (HH:MM:SS) if N/A
    let elapsed_secs = block.get("out_time_us")
        .or_else(|| block.get("out_time_ms"))
        .and_then(|s| {
            let trimmed = s.trim();
            if trimmed.starts_with("N/A") { return None; }
            trimmed.parse::<f64>().ok()
        })
        .map(|us| us / 1_000_000.0)
        .or_else(|| block.get("out_time").and_then(|s| parse_out_time(s)))
        .unwrap_or(0.0);

    let speed_str = block
        .get("speed")
        .cloned()
        .unwrap_or_else(|| "N/A".to_string());

    let percent = if duration > 0.0 {
        (elapsed_secs / duration * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };

    let speed_val: f64 = speed_str.trim_end_matches('x').parse().unwrap_or(1.0);
    let eta_seconds = if percent > 0.0 && percent < 100.0 && speed_val > 0.0 {
        duration * (1.0 - percent / 100.0) / speed_val
    } else {
        0.0
    };

    let _ = app.emit(
        "job-progress",
        ProgressEvent {
            job_id: job_id.to_string(),
            phase: phase.to_string(),
            percent,
            speed: speed_str,
            eta_seconds,
        },
    );
}

// ─── FFmpeg arg builders ──────────────────────────────────────────────────────

fn get_hw_codec(sw_codec: &str, hw_accel: &str) -> Option<String> {
    match (sw_codec, hw_accel) {
        ("libx264", "nvenc")        => Some("h264_nvenc".into()),
        ("libx265", "nvenc")        => Some("hevc_nvenc".into()),
        ("libx264", "amf")          => Some("h264_amf".into()),
        ("libx265", "amf")          => Some("hevc_amf".into()),
        ("libx264", "qsv")          => Some("h264_qsv".into()),
        ("libx265", "qsv")          => Some("hevc_qsv".into()),
        ("libx264", "videotoolbox") => Some("h264_videotoolbox".into()),
        ("libx265", "videotoolbox") => Some("hevc_videotoolbox".into()),
        ("libx264", "vaapi")        => Some("h264_vaapi".into()),
        ("libx265", "vaapi")        => Some("hevc_vaapi".into()),
        _ => None,
    }
}

fn map_nvenc_preset(p: &str) -> &'static str {
    match p {
        "ultrafast" | "superfast" => "p1",
        "veryfast"                => "p2",
        "faster"                  => "p3",
        "fast"                    => "p4",
        "medium"                  => "p5",
        "slow"                    => "p6",
        "slower" | "veryslow"     => "p7",
        _                         => "p5",
    }
}

fn map_qsv_preset(p: &str) -> &'static str {
    match p {
        "ultrafast" | "superfast" | "veryfast" | "faster" | "fast" => "fast",
        "slow" | "slower" | "veryslow"                              => "slow",
        _                                                            => "medium",
    }
}

fn build_video_args(req: &ProcessRequest, output: &str) -> Vec<String> {
    let mut args = vec!["-y".to_string()];
    let is_copy = req.video.codec == "copy";
    let hw = req.video.hw_accel.as_str();
    let hw_codec = if is_copy { None } else { get_hw_codec(&req.video.codec, hw) };

    // Hardware decode acceleration flags (before -i)
    if !is_copy {
        match hw {
            "nvenc" => args.extend(["-hwaccel".into(), "cuda".into()]),
            "qsv"   => args.extend(["-hwaccel".into(), "qsv".into()]),
            "videotoolbox" => args.extend(["-hwaccel".into(), "videotoolbox".into()]),
            // VAAPI: SW decode + HW encode path — just set device globally, no hwaccel flags.
            // Frames upload to GPU via hwupload filter in the filter chain.
            "vaapi" => args.extend(["-vaapi_device".into(), req.video.vaapi_device.clone()]),
            _ => {}
        }
    }

    args.extend(["-i".into(), req.input_path.clone()]);

    // Filter chain
    if !is_copy {
        let di = &req.video.deinterlace;
        let do_di = di.enabled && (!di.auto_detect || req.probe.is_interlaced);

        let scale = match req.video.resolution.as_str() {
            "480p"  => Some("scale=-2:480".into()),
            "720p"  => Some("scale=-2:720".into()),
            "1080p" => Some("scale=-2:1080".into()),
            "1440p" => Some("scale=-2:1440".into()),
            "2160p" => Some("scale=-2:2160".into()),
            "custom" => {
                let w = req.video.custom_width.unwrap_or(0);
                let h = req.video.custom_height.unwrap_or(0);
                if w > 0 && h > 0 { Some(format!("scale={}:{}", w, h)) } else { None }
            }
            _ => None,
        };

        let mut filters: Vec<String> = Vec::new();
        if do_di { filters.push(format!("{}=mode=send_frame:parity=auto", di.algorithm)); }
        if let Some(sf) = scale { filters.push(sf); }
        if hw == "vaapi" {
            filters.push("format=nv12".into());
            filters.push("hwupload".into());
        }
        if !filters.is_empty() {
            args.extend(["-vf".into(), filters.join(",")]);
        }
    }

    // Codec + quality
    if is_copy {
        args.extend(["-c:v".into(), "copy".into()]);
    } else if let Some(ref hc) = hw_codec {
        let crf = req.video.crf;
        let ep = req.video.encode_preset.as_deref().unwrap_or("medium");
        match hw {
            "nvenc" => args.extend([
                "-c:v".into(), hc.clone(),
                "-cq".into(), crf.to_string(),
                "-preset".into(), map_nvenc_preset(ep).into(),
            ]),
            "amf" => args.extend([
                "-c:v".into(), hc.clone(),
                "-quality".into(), "quality".into(),
                "-rc".into(), "cqp".into(),
                "-qp_i".into(), crf.to_string(),
                "-qp_p".into(), crf.to_string(),
            ]),
            "qsv" => args.extend([
                "-c:v".into(), hc.clone(),
                "-global_quality".into(), crf.to_string(),
                "-preset".into(), map_qsv_preset(ep).into(),
            ]),
            "videotoolbox" => {
                let q = (100.0 - crf as f64 * 100.0 / 51.0).round() as u32;
                args.extend(["-c:v".into(), hc.clone(), "-q:v".into(), q.to_string()]);
            }
            "vaapi" => args.extend([
                "-c:v".into(), hc.clone(),
                "-qp".into(), crf.to_string(),
            ]),
            _ => {}
        }
    } else {
        match req.video.codec.as_str() {
            "libvp9" => args.extend([
                "-c:v".into(), "libvp9".into(),
                "-crf".into(), req.video.crf.to_string(),
                "-b:v".into(), "0".into(),
                "-deadline".into(), "good".into(),
                "-cpu-used".into(), "4".into(),
            ]),
            "libsvtav1" => args.extend([
                "-c:v".into(), "libsvtav1".into(),
                "-crf".into(), req.video.crf.to_string(),
                "-preset".into(), "8".into(),
            ]),
            codec => {
                args.extend(["-c:v".into(), codec.into(), "-crf".into(), req.video.crf.to_string()]);
                if let Some(p) = &req.video.encode_preset {
                    args.extend(["-preset".into(), p.clone()]);
                }
            }
        }
    }

    // Title metadata
    if !req.title.is_empty() {
        args.extend(["-metadata".into(), format!("title={}", req.title)]);
    }

    // Audio passthrough + progress
    args.extend(["-c:a".into(), "copy".into()]);
    args.extend(["-progress".into(), "pipe:1".into(), "-nostats".into()]);
    args.push(output.to_string());
    args
}

fn build_audio_args(input: &str, cfg: &AudioExportConfig, output: &str) -> Vec<String> {
    let mut args = vec![
        "-y".to_string(),
        "-i".to_string(),
        input.to_string(),
        "-vn".to_string(),
    ];

    let codec = match cfg.format.as_str() {
        "mp3" => "libmp3lame",
        "m4a" => "aac",
        "ogg" => "libvorbis",
        "opus" => "libopus",
        "flac" => "flac",
        "wav" => match cfg.bit_depth {
            Some(24) => "pcm_s24le",
            Some(32) => "pcm_f32le",
            _ => "pcm_s16le",
        },
        "aiff" => match cfg.bit_depth {
            Some(24) => "pcm_s24be",
            _ => "pcm_s16be",
        },
        _ => "pcm_s16le",
    };

    args.extend(["-c:a".to_string(), codec.to_string()]);

    if !matches!(cfg.format.as_str(), "wav" | "flac" | "aiff") {
        if let Some(br) = &cfg.bitrate {
            args.extend(["-b:a".to_string(), br.clone()]);
        }
    }

    args.extend([
        "-ar".to_string(),
        cfg.sample_rate.to_string(),
        "-ac".to_string(),
        cfg.channels.to_string(),
        "-progress".to_string(),
        "pipe:1".to_string(),
        "-nostats".to_string(),
    ]);

    args.push(output.to_string());
    args
}

// ─── cancel_job ───────────────────────────────────────────────────────────────

#[tauri::command]
pub fn cancel_job(job_id: String, job_store: State<'_, JobStore>) -> Result<(), String> {
    let mut store = job_store.lock().map_err(|e| e.to_string())?;
    for key in [job_id.clone(), format!("{}-audio", job_id)] {
        if let Some(child) = store.remove(&key) {
            let _ = child.kill();
        }
    }
    Ok(())
}

// ─── reveal_in_folder ─────────────────────────────────────────────────────────

#[tauri::command]
pub fn reveal_in_folder(path: String) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        // xdg-open opens the parent directory
        let dir = std::path::Path::new(&path)
            .parent()
            .and_then(|p| p.to_str())
            .unwrap_or("/");
        std::process::Command::new("xdg-open")
            .arg(dir)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .args(["-R", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .args(["/select,", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
