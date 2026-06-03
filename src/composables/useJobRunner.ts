import { onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useJobsStore } from '@/stores/jobs'
import type { VideoProbeResult } from '@/types/preset'
import type { JobProgressPayload, JobCompletePayload, JobErrorPayload } from '@/types/jobs'

export interface StartProcessArgs {
  inputPath: string
  video: object
  audioExport: object | null
  outputConfig: object
  probe: VideoProbeResult
  title: string
}

export function useJobRunner() {
  const jobsStore = useJobsStore()
  const unlisteners: UnlistenFn[] = []

  onMounted(async () => {
    unlisteners.push(
      await listen<JobProgressPayload>('job-progress', e => jobsStore.updateProgress(e.payload)),
      await listen<JobCompletePayload>('job-complete', e => jobsStore.complete(e.payload)),
      await listen<JobErrorPayload>('job-error', e => jobsStore.fail(e.payload)),
    )
  })

  onUnmounted(() => unlisteners.forEach(fn => fn()))

  async function probeVideo(inputPath: string): Promise<VideoProbeResult> {
    return invoke<VideoProbeResult>('probe_video', { inputPath })
  }

  async function startProcess(args: StartProcessArgs) {
    const jobId = crypto.randomUUID()
    jobsStore.addJob(jobId, args.inputPath)
    await invoke('start_process', {
      request: {
        input_path: args.inputPath,
        video: args.video,
        audio_export: args.audioExport,
        output_config: args.outputConfig,
        job_id: jobId,
        probe: args.probe,
        title: args.title,
      },
    })
    return jobId
  }

  async function cancelJob(jobId: string) {
    jobsStore.cancel(jobId)
    await invoke('cancel_job', { jobId }).catch(() => {})
  }

  return { probeVideo, startProcess, cancelJob }
}
