import { open } from '@tauri-apps/plugin-dialog'

export function useFileDialog() {
  async function pickVideoFile(): Promise<string | null> {
    const result = await open({
      multiple: false,
      filters: [
        {
          name: 'Video',
          extensions: ['mp4', 'mkv', 'mov', 'avi', 'webm', 'ts', 'mts', 'm2ts', 'wmv', 'flv', 'f4v'],
        },
      ],
    })
    return typeof result === 'string' ? result : null
  }

  async function pickOutputDir(): Promise<string | null> {
    const result = await open({ directory: true, multiple: false })
    return typeof result === 'string' ? result : null
  }

  return { pickVideoFile, pickOutputDir }
}
