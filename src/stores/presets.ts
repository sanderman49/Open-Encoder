import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { load } from '@tauri-apps/plugin-store'
import type { Preset, VideoConfig, AudioExportConfig, OutputConfig } from '@/types/preset'
import { BUILTIN_PRESETS, DEFAULT_VIDEO_CONFIG, DEFAULT_OUTPUT_CONFIG } from '@/types/preset'

function backfillVideo(v: Partial<VideoConfig>): VideoConfig {
  return {
    ...DEFAULT_VIDEO_CONFIG,
    ...v,
    deinterlace: { ...DEFAULT_VIDEO_CONFIG.deinterlace, ...(v?.deinterlace ?? {}) },
  }
}

// Allowed values — anything outside these means the config is malformed/tampered.
const VALID = {
  codec: ['libx264', 'libx265', 'libvp9', 'libsvtav1', 'copy'],
  container: ['mp4', 'mkv', 'webm', 'mov'],
  resolution: ['480p', '720p', '1080p', '1440p', '2160p', 'source', 'custom'],
  framerate: ['source', '23.976', '24', '25', '29.97', '30', '50', '59.94', '60'],
  hwAccel: ['none', 'nvenc', 'amf', 'qsv', 'videotoolbox', 'vaapi'],
  algorithm: ['yadif', 'bwdif', 'estdif'],
  encodePreset: ['ultrafast', 'superfast', 'veryfast', 'faster', 'fast', 'medium', 'slow', 'slower', 'veryslow'],
  audioFormat: ['mp3', 'wav', 'flac', 'm4a', 'ogg', 'opus', 'aiff'],
  sampleRate: [22050, 44100, 48000, 96000, 192000],
  channels: [1, 2],
} as const

function isObj(x: unknown): x is Record<string, unknown> {
  return typeof x === 'object' && x !== null && !Array.isArray(x)
}

// Reject structurally invalid or impossible configs (e.g. nothing to export).
// A rejected preset is simply not loaded.
function validatePreset(p: Preset): boolean {
  if (!isObj(p) || typeof p.id !== 'string' || typeof p.name !== 'string') return false

  const v = p.video as VideoConfig
  if (!isObj(v)) return false
  if (typeof v.videoEnabled !== 'boolean') return false
  if (!VALID.codec.includes(v.codec as never)) return false
  if (!VALID.container.includes(v.container as never)) return false
  if (!VALID.resolution.includes(v.resolution as never)) return false
  if (!VALID.framerate.includes(v.framerate as never)) return false
  if (!VALID.hwAccel.includes(v.hwAccel as never)) return false
  if (typeof v.crf !== 'number' || v.crf < 0 || v.crf > 51) return false
  if (v.encodePreset !== null && !VALID.encodePreset.includes(v.encodePreset as never)) return false
  const di = v.deinterlace
  if (!isObj(di) || typeof di.enabled !== 'boolean' || typeof di.autoDetect !== 'boolean'
      || !VALID.algorithm.includes(di.algorithm as never)) return false

  const a = p.audioExport
  if (a !== null) {
    if (!isObj(a)) return false
    if (!VALID.audioFormat.includes(a.format as never)) return false
    if (!VALID.sampleRate.includes(a.sampleRate as never)) return false
    if (!VALID.channels.includes(a.channels as never)) return false
  }

  // Core invariant: a job must produce something.
  if (!v.videoEnabled && a === null) return false

  return true
}

export const usePresetsStore = defineStore('presets', () => {
  const userPresets = ref<Preset[]>([])
  const activePresetId = ref<string>(BUILTIN_PRESETS[0].id)

  const currentConfig = ref<{ video: VideoConfig; audioExport: AudioExportConfig | null; output: OutputConfig }>({
    video: { ...DEFAULT_VIDEO_CONFIG, deinterlace: { ...DEFAULT_VIDEO_CONFIG.deinterlace } },
    audioExport: null,
    output: { ...DEFAULT_OUTPUT_CONFIG },
  })

  const allPresets = computed(() => [...BUILTIN_PRESETS, ...userPresets.value])

  const activePreset = computed(
    () => allPresets.value.find(p => p.id === activePresetId.value) ?? allPresets.value[0],
  )

  function applyPreset(preset: Preset, save = false) {
    activePresetId.value = preset.id
    currentConfig.value = {
      video: JSON.parse(JSON.stringify(preset.video)),
      audioExport: preset.audioExport ? JSON.parse(JSON.stringify(preset.audioExport)) : null,
      output: JSON.parse(JSON.stringify(preset.output ?? DEFAULT_OUTPUT_CONFIG)),
    }
    if (save) persist()
  }

  async function load_() {
    try {
      const store = await load('presets.json', { autoSave: false } as Parameters<typeof load>[1])
      const saved = await store.get<Preset[]>('presets')
      if (saved) {
        const normalized = saved.map(p => ({
          ...p,
          video: backfillVideo(p.video as Partial<VideoConfig>),
          output: { ...DEFAULT_OUTPUT_CONFIG, ...(p.output ?? {}) },
        }))
        const valid = normalized.filter(validatePreset)
        if (valid.length !== normalized.length) {
          console.warn(`Skipped ${normalized.length - valid.length} invalid preset(s) in presets.json`)
        }
        userPresets.value = valid
      }
      const savedId = await store.get<string>('activePresetId')
      const target = savedId ? allPresets.value.find(p => p.id === savedId) : null
      applyPreset(target ?? allPresets.value[0])
    } catch {
      // First run — no store yet
      applyPreset(allPresets.value[0])
    }
  }

  async function saveCurrentAsPreset(name: string) {
    const existing = userPresets.value.find(p => p.name === name)
    const preset: Preset = {
      id: existing?.id ?? crypto.randomUUID(),
      name,
      video: JSON.parse(JSON.stringify(currentConfig.value.video)),
      audioExport: currentConfig.value.audioExport
        ? JSON.parse(JSON.stringify(currentConfig.value.audioExport))
        : null,
      output: JSON.parse(JSON.stringify(currentConfig.value.output)),
    }
    if (existing) {
      const idx = userPresets.value.indexOf(existing)
      userPresets.value[idx] = preset
    } else {
      userPresets.value.push(preset)
    }
    activePresetId.value = preset.id
    await persist()
  }

  async function deletePreset(id: string) {
    userPresets.value = userPresets.value.filter(p => p.id !== id)
    if (activePresetId.value === id) applyPreset(allPresets.value[0])
    await persist()
  }

  async function renamePreset(id: string, name: string) {
    const preset = userPresets.value.find(p => p.id === id)
    if (!preset) return
    preset.name = name.trim()
    await persist()
  }

  async function persist() {
    try {
      const store = await load('presets.json', { autoSave: false } as Parameters<typeof load>[1])
      await store.set('presets', userPresets.value)
      await store.set('activePresetId', activePresetId.value)
      await store.save()
    } catch (e) {
      console.error('Failed to save presets:', e)
    }
  }

  return {
    userPresets,
    allPresets,
    activePresetId,
    activePreset,
    currentConfig,
    load: load_,
    applyPreset,
    saveCurrentAsPreset,
    deletePreset,
    renamePreset,
  }
})
