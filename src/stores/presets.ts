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
      if (saved) userPresets.value = saved.map(p => ({
        ...p,
        video: backfillVideo(p.video as Partial<VideoConfig>),
        output: { ...DEFAULT_OUTPUT_CONFIG, ...(p.output ?? {}) },
      }))
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
