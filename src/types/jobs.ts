export type JobStatus = 'queued' | 'running' | 'completed' | 'failed' | 'cancelled'

export interface Job {
  id: string
  status: JobStatus
  inputPath: string
  videoOutput: string
  audioOutput?: string
  videoPercent: number
  audioPercent: number
  speed: string
  etaSeconds: number
  activePhase: 'video' | 'audio'
  error?: string
  createdAt: number
  completedAt?: number
}

export interface JobProgressPayload {
  job_id: string
  phase: 'video' | 'audio'
  percent: number
  speed: string
  eta_seconds: number
}

export interface JobCompletePayload {
  job_id: string
  video_output: string
  audio_output?: string
}

export interface JobErrorPayload {
  job_id: string
  error: string
}
