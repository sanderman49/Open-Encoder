import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Job, JobProgressPayload, JobCompletePayload, JobErrorPayload } from '@/types/jobs'

export const useJobsStore = defineStore('jobs', () => {
  const jobs = ref<Job[]>([])

  const activeJobs = computed(() =>
    jobs.value.filter(j => j.status === 'queued' || j.status === 'running'),
  )
  const completedJobs = computed(() =>
    jobs.value
      .filter(j => j.status === 'completed' || j.status === 'failed' || j.status === 'cancelled')
      .slice()
      .sort((a, b) => (b.completedAt ?? 0) - (a.completedAt ?? 0)),
  )

  function addJob(id: string, inputPath: string) {
    jobs.value.push({
      id,
      status: 'queued',
      inputPath,
      videoOutput: '',
      videoPercent: 0,
      audioPercent: 0,
      speed: 'N/A',
      etaSeconds: 0,
      activePhase: 'video',
      createdAt: Date.now(),
    })
  }

  function updateProgress(payload: JobProgressPayload) {
    const job = jobs.value.find(j => j.id === payload.job_id)
    if (!job) return
    job.status = 'running'
    job.speed = payload.speed
    job.etaSeconds = payload.eta_seconds
    job.activePhase = payload.phase
    if (payload.phase === 'video') job.videoPercent = payload.percent
    else job.audioPercent = payload.percent
  }

  function complete(payload: JobCompletePayload) {
    const job = jobs.value.find(j => j.id === payload.job_id)
    if (!job) return
    job.status = 'completed'
    job.videoOutput = payload.video_output
    job.audioOutput = payload.audio_output
    job.videoPercent = 100
    job.audioPercent = payload.audio_output ? 100 : 0
    job.completedAt = Date.now()
  }

  function fail(payload: JobErrorPayload) {
    const job = jobs.value.find(j => j.id === payload.job_id)
    if (!job) return
    job.status = 'failed'
    job.error = payload.error
    job.completedAt = Date.now()
  }

  function cancel(id: string) {
    const job = jobs.value.find(j => j.id === id)
    if (!job) return
    job.status = 'cancelled'
    job.completedAt = Date.now()
  }

  return { jobs, activeJobs, completedJobs, addJob, updateProgress, complete, fail, cancel }
})
