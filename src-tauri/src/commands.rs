use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_shell::process::{CommandChild, CommandEvent};
use tauri_plugin_shell::ShellExt;

use crate::JobStore;

// ─── Request types ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
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
pub struct ProcessRequest {
    pub input_path: String,
    pub output_dir: String,
    pub video: VideoConfig,
    pub audio_export: Option<AudioExportConfig>,
    pub job_id: String,
    pub probe: VideoProbeResult,
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

async fn run_process(
    app: AppHandle,
    req: ProcessRequest,
    job_store: Arc<Mutex<HashMap<String, CommandChild>>>,
) -> Result<(), String> {
    let stem = Path::new(&req.input_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output")
        .to_string();

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

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

    let video_out = format!("{}/{}_{}_{}.{}", req.output_dir, stem, ts, "processed", ext);
    let video_args = build_video_args(&req, &video_out);

    let (mut rx, child) = app
        .shell()
        .sidecar("binaries/ffmpeg")
        .map_err(|e| e.to_string())?
        .args(&video_args)
        .spawn()
        .map_err(|e| e.to_string())?;

    job_store.lock().unwrap().insert(req.job_id.clone(), child);

    // Stream video progress
    {
        let mut line_buf = String::new();
        let mut block: HashMap<String, String> = HashMap::new();
        loop {
            match rx.recv().await {
                Some(CommandEvent::Stderr(b)) => {
                    line_buf.push_str(&String::from_utf8_lossy(&b));
                    while let Some(nl) = line_buf.find('\n') {
                        let line = line_buf[..nl].trim().to_string();
                        line_buf.drain(..=nl);
                        if line.is_empty() {
                            continue;
                        }
                        if let Some((k, v)) = line.split_once('=') {
                            block.insert(k.to_string(), v.trim().to_string());
                        }
                        if line.starts_with("progress=") {
                            emit_progress(&app, &block, req.probe.duration, &req.job_id, "video");
                            block.clear();
                        }
                    }
                }
                Some(CommandEvent::Terminated(p)) => {
                    job_store.lock().unwrap().remove(&req.job_id);
                    if p.code != Some(0) {
                        return Err(format!("FFmpeg (video) exited with code {:?}", p.code));
                    }
                    break;
                }
                None => {
                    job_store.lock().unwrap().remove(&req.job_id);
                    break;
                }
                _ => {}
            }
        }
    }

    // ── Audio export pass ────────────────────────────────────────────────────
    let audio_out = if let Some(ref cfg) = req.audio_export {
        let out = format!("{}/{}_{}_{}.{}", req.output_dir, stem, ts, "audio", cfg.format);
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
        loop {
            match rx2.recv().await {
                Some(CommandEvent::Stderr(b)) => {
                    line_buf2.push_str(&String::from_utf8_lossy(&b));
                    while let Some(nl) = line_buf2.find('\n') {
                        let line = line_buf2[..nl].trim().to_string();
                        line_buf2.drain(..=nl);
                        if line.is_empty() {
                            continue;
                        }
                        if let Some((k, v)) = line.split_once('=') {
                            block2.insert(k.to_string(), v.trim().to_string());
                        }
                        if line.starts_with("progress=") {
                            emit_progress(&app, &block2, req.probe.duration, &req.job_id, "audio");
                            block2.clear();
                        }
                    }
                }
                Some(CommandEvent::Terminated(p)) => {
                    job_store.lock().unwrap().remove(&audio_key);
                    if p.code != Some(0) {
                        return Err(format!("FFmpeg (audio) exited with code {:?}", p.code));
                    }
                    break;
                }
                None => {
                    job_store.lock().unwrap().remove(&audio_key);
                    break;
                }
                _ => {}
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

fn emit_progress(
    app: &AppHandle,
    block: &HashMap<String, String>,
    duration: f64,
    job_id: &str,
    phase: &str,
) {
    let out_time_ms: f64 = block
        .get("out_time_ms")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);

    let speed_str = block
        .get("speed")
        .cloned()
        .unwrap_or_else(|| "N/A".to_string());

    let percent = if duration > 0.0 {
        ((out_time_ms / 1_000_000.0) / duration * 100.0)
            .clamp(0.0, 100.0)
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

fn build_video_args(req: &ProcessRequest, output: &str) -> Vec<String> {
    let mut args = vec![
        "-y".to_string(),
        "-i".to_string(),
        req.input_path.clone(),
    ];

    let di = &req.video.deinterlace;
    let do_deinterlace =
        req.video.codec != "copy" && di.enabled && (!di.auto_detect || req.probe.is_interlaced);

    let scale_filter = if req.video.codec != "copy" {
        match req.video.resolution.as_str() {
            "480p" => Some("scale=-2:480".to_string()),
            "720p" => Some("scale=-2:720".to_string()),
            "1080p" => Some("scale=-2:1080".to_string()),
            "1440p" => Some("scale=-2:1440".to_string()),
            "2160p" => Some("scale=-2:2160".to_string()),
            "custom" => {
                let w = req.video.custom_width.unwrap_or(0);
                let h = req.video.custom_height.unwrap_or(0);
                if w > 0 && h > 0 {
                    Some(format!("scale={}:{}", w, h))
                } else {
                    None
                }
            }
            _ => None,
        }
    } else {
        None
    };

    // Build filter chain
    if req.video.codec != "copy" {
        let mut filters: Vec<String> = Vec::new();
        if do_deinterlace {
            filters.push(format!("{}=mode=send_frame:parity=auto", di.algorithm));
        }
        if let Some(sf) = scale_filter {
            filters.push(sf);
        }
        if !filters.is_empty() {
            args.extend(["-vf".to_string(), filters.join(",")]);
        }
    }

    // Codec
    match req.video.codec.as_str() {
        "copy" => args.extend(["-c:v".to_string(), "copy".to_string()]),
        "libvp9" => args.extend([
            "-c:v".to_string(),
            "libvp9".to_string(),
            "-crf".to_string(),
            req.video.crf.to_string(),
            "-b:v".to_string(),
            "0".to_string(),
            "-deadline".to_string(),
            "good".to_string(),
            "-cpu-used".to_string(),
            "4".to_string(),
        ]),
        "libsvtav1" => args.extend([
            "-c:v".to_string(),
            "libsvtav1".to_string(),
            "-crf".to_string(),
            req.video.crf.to_string(),
            "-preset".to_string(),
            "8".to_string(),
        ]),
        codec => {
            args.extend([
                "-c:v".to_string(),
                codec.to_string(),
                "-crf".to_string(),
                req.video.crf.to_string(),
            ]);
            if let Some(p) = &req.video.encode_preset {
                args.extend(["-preset".to_string(), p.clone()]);
            }
        }
    }

    // Always passthrough audio
    args.extend(["-c:a".to_string(), "copy".to_string()]);

    // Progress reporting to stderr
    args.extend([
        "-progress".to_string(),
        "pipe:2".to_string(),
        "-nostats".to_string(),
    ]);

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
        "pipe:2".to_string(),
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
