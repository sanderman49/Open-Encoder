<script setup lang="ts">
import { ref } from 'vue'
import { useJobRunner } from '@/composables/useJobRunner'
import { useFileDialog } from '@/composables/useFileDialog'
import type { VideoProbeResult } from '@/types/preset'

const emit = defineEmits<{
  fileSelected: [path: string, probe: VideoProbeResult]
}>()

const { probeVideo } = useJobRunner()
const { pickVideoFile } = useFileDialog()

const dragging = ref(false)
const loading = ref(false)
const error = ref<string | null>(null)
const file = ref<string | null>(null)
const probe = ref<VideoProbeResult | null>(null)

async function handlePath(path: string) {
  file.value = null
  probe.value = null
  error.value = null
  loading.value = true
  try {
    const result = await probeVideo(path)
    if (!result.hasVideo) throw new Error('No video stream found')
    file.value = path
    probe.value = result
    emit('fileSelected', path, result)
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : String(e)
  } finally {
    loading.value = false
  }
}

async function onDrop(e: DragEvent) {
  dragging.value = false
  const path = e.dataTransfer?.files?.[0]?.path
  if (path) await handlePath(path)
}

async function onBrowse() {
  const path = await pickVideoFile()
  if (path) await handlePath(path)
}

function formatDuration(s: number) {
  const h = Math.floor(s / 3600)
  const m = Math.floor((s % 3600) / 60)
  const sec = Math.floor(s % 60)
  return h > 0
    ? `${h}:${String(m).padStart(2, '0')}:${String(sec).padStart(2, '0')}`
    : `${m}:${String(sec).padStart(2, '0')}`
}

function basename(path: string) {
  return path.split(/[/\\]/).pop() ?? path
}
</script>

<template>
  <div
    class="dropzone"
    :class="{ 'dropzone--drag': dragging, 'dropzone--loaded': !!probe }"
    @dragover.prevent="dragging = true"
    @dragleave="dragging = false"
    @drop.prevent="onDrop"
    @click="onBrowse"
  >
    <template v-if="loading">
      <div class="dropzone__icon">⏳</div>
      <p class="dropzone__label">Probing…</p>
    </template>

    <template v-else-if="probe && file">
      <div class="dropzone__info">
        <span class="file-name">{{ basename(file) }}</span>
        <div class="meta">
          <span class="meta-item">{{ formatDuration(probe.duration) }}</span>
          <span v-if="probe.width && probe.height" class="meta-item">{{ probe.width }}×{{ probe.height }}</span>
          <span v-if="probe.videoCodec" class="meta-item">{{ probe.videoCodec }}</span>
          <span v-if="probe.audioCodec" class="meta-item">{{ probe.audioCodec }}</span>
        </div>
        <div v-if="probe.isInterlaced" class="badge badge--warn">
          ⚠ Interlaced ({{ probe.fieldOrder }})
        </div>
        <p class="dropzone__hint">Click or drop to change file</p>
      </div>
    </template>

    <template v-else>
      <div class="dropzone__icon">🎬</div>
      <p class="dropzone__label">Drop video here or click to browse</p>
      <p class="dropzone__sub">MP4, MKV, MOV, AVI, WebM, TS, MTS…</p>
    </template>

    <div v-if="error" class="dropzone__error">{{ error }}</div>
  </div>
</template>

<style scoped>
.dropzone {
  border: 2px dashed var(--border);
  border-radius: var(--radius);
  padding: 32px 24px;
  text-align: center;
  cursor: pointer;
  transition: border-color 0.2s, background 0.2s;
  min-height: 140px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
}
.dropzone:hover,
.dropzone--drag { border-color: var(--accent); background: var(--accent-dim); }
.dropzone--loaded { border-style: solid; border-color: var(--border); }

.dropzone__icon { font-size: 32px; }
.dropzone__label { font-size: 15px; font-weight: 500; }
.dropzone__sub { color: var(--muted); font-size: 12px; }
.dropzone__hint { color: var(--muted); font-size: 11px; margin-top: 4px; }
.dropzone__error {
  color: var(--danger);
  font-size: 12px;
  margin-top: 8px;
}

.dropzone__info { display: flex; flex-direction: column; align-items: center; gap: 6px; }
.file-name { font-weight: 600; font-size: 15px; word-break: break-all; }

.meta {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  justify-content: center;
}
.meta-item {
  background: var(--elevated);
  border: 1px solid var(--border);
  border-radius: 4px;
  padding: 2px 8px;
  font-size: 12px;
  color: var(--muted);
}

.badge {
  font-size: 12px;
  padding: 3px 10px;
  border-radius: 4px;
  font-weight: 500;
}
.badge--warn {
  background: rgba(245, 158, 11, 0.15);
  color: var(--warning);
  border: 1px solid rgba(245, 158, 11, 0.3);
}
</style>
