export type VideoCodec = 'libx264' | 'libx265' | 'libvp9' | 'libsvtav1' | 'copy'
export type HwAccel = 'none' | 'nvenc' | 'amf' | 'qsv' | 'videotoolbox' | 'vaapi'
export type Container = 'mp4' | 'mkv' | 'webm' | 'mov'
export type Resolution = '480p' | '720p' | '1080p' | '1440p' | '2160p' | 'source' | 'custom'
export type Framerate = 'source' | '23.976' | '24' | '25' | '29.97' | '30' | '50' | '59.94' | '60'
export type DeinterlaceAlgo = 'yadif' | 'bwdif' | 'estdif'
export type AudioFormat = 'mp3' | 'wav' | 'flac' | 'm4a' | 'ogg' | 'opus' | 'aiff'
export type SampleRate = 22050 | 44100 | 48000 | 96000 | 192000
export type BitDepth = 16 | 24 | 32
export type EncodePreset = 'ultrafast' | 'superfast' | 'veryfast' | 'faster' | 'fast' | 'medium' | 'slow' | 'slower' | 'veryslow'

export interface DeinterlaceConfig {
  enabled: boolean
  autoDetect: boolean
  algorithm: DeinterlaceAlgo
}

export interface VideoConfig {
  videoEnabled: boolean
  codec: VideoCodec
  container: Container
  resolution: Resolution
  customWidth?: number
  customHeight?: number
  framerate: Framerate
  crf: number
  encodePreset: EncodePreset | null
  deinterlace: DeinterlaceConfig
  hwAccel: HwAccel
  vaapiDevice: string
}

export interface AudioExportConfig {
  format: AudioFormat
  bitrate?: string
  sampleRate: SampleRate
  bitDepth?: BitDepth
  channels: 1 | 2
}

export interface OutputConfig {
  videoDir: string          // base output dir (absolute); supports variables; empty = input dir
  videoSubdir: string       // subfolder within videoDir for video; empty = directly in videoDir
  audioDir: string          // subfolder within videoDir for audio; empty = same as videoDir
  nameOverride: string      // video filename stem override; supports variables; empty = use title
  audioNameOverride: string // audio filename stem override; supports variables; empty = inherit video stem
}

export interface Preset {
  id: string
  name: string
  video: VideoConfig
  audioExport: AudioExportConfig | null
  output: OutputConfig
}

export interface VideoProbeResult {
  duration: number
  hasAudio: boolean
  hasVideo: boolean
  width?: number
  height?: number
  videoCodec?: string
  audioCodec?: string
  isInterlaced: boolean
  fieldOrder?: string
}

// Valid containers per codec
export const CODEC_CONTAINERS: Record<VideoCodec, Container[]> = {
  libx264:  ['mp4', 'mkv', 'mov'],
  libx265:  ['mp4', 'mkv', 'mov'],
  libvp9:   ['webm', 'mkv'],
  libsvtav1:['mkv', 'mp4'],
  copy:     ['mp4', 'mkv', 'webm', 'mov'],
}

export const DEFAULT_VIDEO_CONFIG: VideoConfig = {
  videoEnabled: true,
  codec: 'libx264',
  container: 'mp4',
  resolution: 'source',
  framerate: 'source',
  crf: 23,
  encodePreset: 'fast',
  deinterlace: { enabled: false, autoDetect: true, algorithm: 'bwdif' },
  hwAccel: 'none',
  vaapiDevice: '/dev/dri/renderD128',
}

export const DEFAULT_AUDIO_EXPORT_CONFIG: AudioExportConfig = {
  format: 'mp3',
  bitrate: '320k',
  sampleRate: 48000,
  channels: 2,
}

export const DEFAULT_OUTPUT_CONFIG: OutputConfig = {
  videoDir: '',
  videoSubdir: '',
  audioDir: '',
  nameOverride: '',
  audioNameOverride: '',
}

function makePreset(
  id: string,
  name: string,
  resolution: Resolution,
): Preset {
  return {
    id,
    name,
    video: {
      videoEnabled: true,
      codec: 'libx264',
      container: 'mp4',
      resolution,
      framerate: 'source',
      crf: 18,
      encodePreset: 'slow',
      deinterlace: { enabled: true, autoDetect: true, algorithm: 'bwdif' },
      hwAccel: 'none',
      vaapiDevice: '/dev/dri/renderD128',
    },
    audioExport: { format: 'mp3', bitrate: '320k', sampleRate: 48000, channels: 2 },
    output: { ...DEFAULT_OUTPUT_CONFIG },
  }

}

export const BUILTIN_PRESETS: Preset[] = [
  makePreset('builtin-720p',  '720p',  '720p'),
  makePreset('builtin-1080p', '1080p', '1080p'),
  makePreset('builtin-1440p', '1440p', '1440p'),
  makePreset('builtin-4k',    '4K',    '2160p'),
]
