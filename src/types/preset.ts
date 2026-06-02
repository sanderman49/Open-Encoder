export type VideoCodec = 'libx264' | 'libx265' | 'libvp9' | 'libsvtav1' | 'copy'
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
}

export interface AudioExportConfig {
  format: AudioFormat
  bitrate?: string
  sampleRate: SampleRate
  bitDepth?: BitDepth
  channels: 1 | 2
}

export interface Preset {
  id: string
  name: string
  video: VideoConfig
  audioExport: AudioExportConfig | null
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
}

export const BUILTIN_PRESETS: Preset[] = [
  {
    id: 'builtin-yt1440p',
    name: 'YouTube 1440p',
    video: {
      codec: 'libx264',
      container: 'mp4',
      resolution: '1440p',
      crf: 18,
      encodePreset: 'slow',
      deinterlace: { enabled: false, autoDetect: true, algorithm: 'bwdif' },
    },
    audioExport: { format: 'mp3', bitrate: '320k', sampleRate: 48000, channels: 2 },
  },
  {
    id: 'builtin-deinterlace',
    name: 'Deinterlace Only',
    video: {
      codec: 'libx264',
      container: 'mp4',
      resolution: 'source',
      crf: 18,
      encodePreset: 'fast',
      deinterlace: { enabled: true, autoDetect: true, algorithm: 'bwdif' },
    },
    audioExport: null,
  },
  {
    id: 'builtin-podcast',
    name: 'Podcast Audio',
    video: {
      codec: 'copy',
      container: 'mp4',
      resolution: 'source',
      crf: 23,
      encodePreset: null,
      deinterlace: { enabled: false, autoDetect: true, algorithm: 'bwdif' },
    },
    audioExport: { format: 'mp3', bitrate: '128k', sampleRate: 44100, channels: 1 },
  },
]
