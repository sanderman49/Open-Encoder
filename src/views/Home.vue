<script setup lang="ts">
import { ref } from 'vue'
import { usePresetsStore } from '@/stores/presets'
import { useJobsStore } from '@/stores/jobs'
import { useJobRunner } from '@/composables/useJobRunner'
import { useFileDialog } from '@/composables/useFileDialog'
import PresetBar from '@/components/PresetBar.vue'
import DropZone from '@/components/DropZone.vue'
import SettingsDrawer from '@/components/SettingsDrawer.vue'
import JobQueue from '@/components/JobQueue.vue'
import JobHistory from '@/components/JobHistory.vue'
import type { VideoProbeResult } from '@/types/preset'

const presetsStore = usePresetsStore()
const jobsStore = useJobsStore()
const { startProcess, cancelJob } = useJobRunner()
const { pickOutputDir } = useFileDialog()

const drawerOpen = ref(false)
const selectedFile = ref<string | null>(null)
const probeResult = ref<VideoProbeResult | null>(null)
const outputDir = ref<string>('')
const starting = ref(false)
const startError = ref<string | null>(null)

function onFileSelected(path: string, probe: VideoProbeResult) {
  selectedFile.value = path
  probeResult.value = probe
  startError.value = null
}

async function onPickDir() {
  const dir = await pickOutputDir()
  if (dir) outputDir.value = dir
}

function basename(p: string) { return p.split(/[/\\]/).pop() ?? p }

function toRustConfig(cfg: typeof presetsStore.currentConfig) {
  const v = cfg.video
  return {
    video: {
      codec: v.codec,
      container: v.container,
      resolution: v.resolution,
      custom_width: v.customWidth ?? null,
      custom_height: v.customHeight ?? null,
      crf: v.crf,
      encode_preset: v.encodePreset ?? null,
      deinterlace: {
        enabled: v.deinterlace.enabled,
        auto_detect: v.deinterlace.autoDetect,
        algorithm: v.deinterlace.algorithm,
      },
    },
    audioExport: cfg.audioExport
      ? {
          format: cfg.audioExport.format,
          bitrate: cfg.audioExport.bitrate ?? null,
          sample_rate: cfg.audioExport.sampleRate,
          bit_depth: cfg.audioExport.bitDepth ?? null,
          channels: cfg.audioExport.channels,
        }
      : null,
  }
}

async function handleStart() {
  if (!selectedFile.value || !outputDir.value || !probeResult.value) return
  starting.value = true
  startError.value = null
  try {
    const { video, audioExport } = toRustConfig(presetsStore.currentConfig)
    await startProcess({
      inputPath: selectedFile.value,
      outputDir: outputDir.value,
      video,
      audioExport,
      probe: probeResult.value,
    })
  } catch (e: unknown) {
    startError.value = e instanceof Error ? e.message : String(e)
  } finally {
    starting.value = false
  }
}

const canStart = () => !!selectedFile.value && !!outputDir.value && !starting.value
</script>

<template>
  <div class="layout">
    <PresetBar @open-settings="drawerOpen = true" />

    <main class="main">
      <DropZone @file-selected="onFileSelected" />

      <div class="output-row">
        <div class="output-path">
          <span class="output-label">Output folder</span>
          <span class="output-val">{{ outputDir || 'Not selected' }}</span>
        </div>
        <button class="btn btn-ghost" @click="onPickDir">Browse…</button>
      </div>

      <div v-if="startError" class="start-error">{{ startError }}</div>

      <button
        class="btn btn-primary start-btn"
        :disabled="!canStart()"
        @click="handleStart"
      >
        {{ starting ? 'Starting…' : '▶ Start Processing' }}
      </button>

      <JobQueue :jobs="jobsStore.activeJobs" @cancel="cancelJob" />
      <JobHistory :jobs="jobsStore.completedJobs" />
    </main>

    <SettingsDrawer v-model:open="drawerOpen" />
  </div>
</template>

<style scoped>
.layout {
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
}

.main {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.output-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 12px 16px;
}
.output-path { display: flex; flex-direction: column; gap: 2px; min-width: 0; }
.output-label { font-size: 11px; color: var(--muted); }
.output-val {
  font-size: 13px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  color: var(--text);
}

.start-btn {
  width: 100%;
  justify-content: center;
  padding: 12px;
  font-size: 15px;
}

.start-error {
  color: var(--danger);
  font-size: 13px;
  padding: 8px 12px;
  background: rgba(239, 68, 68, 0.08);
  border-radius: var(--radius-sm);
  border: 1px solid rgba(239, 68, 68, 0.2);
}
</style>
