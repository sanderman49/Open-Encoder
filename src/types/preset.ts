export type VideoCodec = 'libx264' | 'libx265' | 'libvp9' | 'libsvtav1' | 'copy'
export type HwAccel = 'none' | 'nvenc' | 'amf' | 'qsv' | 'videotoolbox' | 'vaapi'
export type Container = 'mp4' | 'mkv' | 'webm' | 'mov'
export type Resolution = '480p' | '720p' | '1080p' | '1440p' | '2160p' | 'source' | 'custom'
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
  codec: VideoCodec
  container: Container
  resolution: Resolution
  customWidth?: number
  customHeight?: number
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
  videoDir: string           // absolute path; empty = input file's directory
  audioDir: string           // path for audio output; empty = same as resolved video dir
  audioDirRelative: boolean  // if true, audioDir is relative to input file's directory
  createDateFolder: boolean  // create YYYY-MM-DD subfolder inside output dir
  filenamePrefix: string     // prepend to output filename stem
  filenameSuffix: string     // append to output filename stem (before extension)
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
  codec: 'libx264',
  container: 'mp4',
  resolution: 'source',
  crf: 23,
  encodePreset: 'medium',
  deinterlace: { enabled: false, autoDetect: true, algorithm: 'bwdif' },
  hwAccel: 'none',
  vaapiDevice: '/dev/dri/renderD128',
}

export const DEFAULT_OUTPUT_CONFIG: OutputConfig = {
  videoDir: '',
  audioDir: '',
  audioDirRelative: false,
  createDateFolder: false,
  filenamePrefix: '',
  filenameSuffix: '',
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
      codec: 'libx264',
      container: 'mp4',
      resolution,
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
