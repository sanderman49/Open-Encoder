<script setup lang="ts">
import { ref, computed } from 'vue'
import { usePresetsStore } from '@/stores/presets'
import { useJobsStore } from '@/stores/jobs'
import { useJobRunner } from '@/composables/useJobRunner'
import { useFileDialog } from '@/composables/useFileDialog'
import PresetBar from '@/components/PresetBar.vue'
import DropZone from '@/components/DropZone.vue'
import SettingsDrawer from '@/components/SettingsDrawer.vue'
import JobQueue from '@/components/JobQueue.vue'
import JobHistory from '@/components/JobHistory.vue'
import AnimatedDots from '@/components/AnimatedDots.vue'
import type { VideoProbeResult } from '@/types/preset'
import { Play, FolderOpen } from 'lucide-vue-next'

const presetsStore = usePresetsStore()
const jobsStore = useJobsStore()
const { startProcess, cancelJob } = useJobRunner()
const { pickOutputDir } = useFileDialog()

const drawerOpen = ref(false)
const selectedFile = ref<string | null>(null)
const probeResult = ref<VideoProbeResult | null>(null)
const sessionOutputDir = ref<string>('')
const videoTitle = ref<string>('')
const starting = ref(false)
const startError = ref<string | null>(null)

// If preset has a video dir configured, use it and lock the field
const presetVideoDir = computed(() => presetsStore.currentConfig.output.videoDir)
const outputDirLocked = computed(() => presetVideoDir.value.length > 0)
const displayedOutputDir = computed(() =>
  outputDirLocked.value ? presetVideoDir.value : sessionOutputDir.value
)

function onFileSelected(path: string, probe: VideoProbeResult) {
  selectedFile.value = path
  probeResult.value = probe
  startError.value = null
  // Default title to the input filename stem
  videoTitle.value = path.split(/[/\\]/).pop()?.replace(/\.[^.]+$/, '') ?? ''
}

async function onPickDir() {
  const dir = await pickOutputDir()
  if (dir) sessionOutputDir.value = dir
}

function toRustConfig(cfg: typeof presetsStore.currentConfig) {
  const v = cfg.video
  const o = cfg.output
  // If preset doesn't specify a video dir, use the session-selected one
  const videoDir = o.videoDir || sessionOutputDir.value
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
      hw_accel: v.hwAccel ?? 'none',
      vaapi_device: v.vaapiDevice ?? '/dev/dri/renderD128',
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
    outputConfig: {
      video_dir: videoDir,
      audio_dir: o.audioDir,
      audio_dir_relative: o.audioDirRelative,
      create_date_folder: o.createDateFolder,
      filename_prefix: o.filenamePrefix,
      filename_suffix: o.filenameSuffix,
    },
  }
}

async function handleStart() {
  if (!selectedFile.value || !probeResult.value) return
  starting.value = true
  startError.value = null
  try {
    const { video, audioExport, outputConfig } = toRustConfig(presetsStore.currentConfig)
    await startProcess({
      inputPath: selectedFile.value,
      video,
      audioExport,
      outputConfig,
      probe: probeResult.value,
      title: videoTitle.value,
    })
  } catch (e: unknown) {
    startError.value = e instanceof Error ? e.message : String(e)
  } finally {
    starting.value = false
  }
}

const canStart = () =>
  !!selectedFile.value && !starting.value && jobsStore.activeJobs.length === 0
</script>

<template>
  <div class="layout">
    <PresetBar @open-settings="drawerOpen = true" />

    <main class="main">
      <DropZone @file-selected="onFileSelected" />

      <div class="title-row">
        <span class="title-fix" v-if="presetsStore.currentConfig.output.filenamePrefix">{{ presetsStore.currentConfig.output.filenamePrefix }}</span>
        <input
          v-model="videoTitle"
          class="title-input"
          placeholder="Title / filename"
        />
        <span class="title-fix" v-if="presetsStore.currentConfig.output.filenameSuffix">{{ presetsStore.currentConfig.output.filenameSuffix }}</span>
      </div>

      <div class="output-row">
        <div class="output-path">
          <span class="output-label">Output folder</span>
          <span class="output-val">
            {{ displayedOutputDir || 'Same as original video' }}
          </span>
        </div>
        <div class="output-right">
          <span v-if="outputDirLocked" class="locked-hint">Set by preset</span>
          <button
            class="btn btn-ghost"
            :disabled="outputDirLocked"
            @click="onPickDir"
            :title="outputDirLocked ? 'Configured in preset settings' : 'Choose output folder'"
          >
            <FolderOpen :size="14" />
            Browse
          </button>
        </div>
      </div>

      <div v-if="startError" class="start-error">{{ startError }}</div>

      <button
        class="btn btn-primary start-btn"
        :disabled="!canStart()"
        @click="handleStart"
      >
        <Play v-if="!jobsStore.activeJobs.length && !starting" :size="16" />
        <span v-if="jobsStore.activeJobs.length">Processing<AnimatedDots /></span>
        <span v-else-if="starting">Starting…</span>
        <span v-else>Start Processing</span>
      </button>

      <JobQueue :jobs="jobsStore.activeJobs" @cancel="cancelJob" />
      <JobHistory :jobs="jobsStore.completedJobs" @clear-history="jobsStore.clearHistory()" />
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

.title-row {
  display: flex;
  align-items: center;
  gap: 0;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  overflow: hidden;
}
.title-fix {
  padding: 10px 12px;
  font-size: 13px;
  color: var(--muted);
  background: var(--elevated);
  white-space: nowrap;
  flex-shrink: 0;
  border-right: 1px solid var(--border);
}
.title-fix:last-child {
  border-right: none;
  border-left: 1px solid var(--border);
}
.title-input {
  flex: 1;
  border: none;
  border-radius: 0;
  background: var(--surface);
  padding: 10px 12px;
  font-size: 13px;
  min-width: 0;
}
.title-input:focus { border-color: transparent; outline: none; }

.output-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 12px 16px;
  transition: opacity 0.15s;
}
.output-path { display: flex; flex-direction: column; gap: 2px; min-width: 0; flex: 1; }
.output-right { display: flex; align-items: center; gap: 8px; flex-shrink: 0; }
.locked-hint { font-size: 11px; color: var(--muted); white-space: nowrap; }
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
