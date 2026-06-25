<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useJobsStore } from '@/stores/jobs'
import type { Job } from '@/types/jobs'

defineProps<{ jobs: Job[] }>()
defineEmits<{ clearHistory: [] }>()

const jobsStore = useJobsStore()
const expanded = ref<Record<string, boolean>>({})

function basename(p: string) { return p.split(/[/\\]/).pop() ?? p }

function dirname(p: string) {
  const parts = p.split(/[/\\]/)
  parts.pop()
  return parts.join('/')
}

async function openFolder(filePath?: string) {
  if (!filePath) return
  await invoke('reveal_in_folder', { path: filePath }).catch(() => {})
}

function formatTime(ms: number) {
  return new Date(ms).toLocaleTimeString()
}
</script>

<template>
  <div v-if="jobs.length" class="history">
    <div class="history-header-row">
      <p class="section-title" style="margin:0">History</p>
      <button class="btn btn-ghost clear-btn" @click="$emit('clearHistory')">Clear</button>
    </div>
    <div v-for="job in jobs" :key="job.id" class="history-card" :class="`history-card--${job.status}`">
      <div class="history-header">
        <span class="history-name">{{ basename(job.inputPath) }}</span>
        <span class="status-badge" :class="`badge--${job.status}`">{{ job.status }}</span>
      </div>

      <template v-if="job.status === 'completed'">
        <div v-if="job.videoOutput" class="output-row" @click="openFolder(job.videoOutput)" title="Open folder">
          <span class="output-icon">🎬</span>
          <span class="output-name">{{ basename(job.videoOutput) }}</span>
          <span class="open-hint">↗</span>
        </div>
        <div v-if="job.audioOutput" class="output-row" @click="openFolder(job.audioOutput)" title="Open folder">
          <span class="output-icon">🎵</span>
          <span class="output-name">{{ basename(job.audioOutput) }}</span>
          <span class="open-hint">↗</span>
        </div>
      </template>

      <div v-if="job.status === 'failed'" class="error-msg">{{ job.error }}</div>

      <div class="history-footer">
        <span class="history-time">{{ formatTime(job.completedAt ?? job.createdAt) }}</span>
        <button v-if="jobsStore.logs[job.id]?.length" class="log-toggle" @click="expanded[job.id] = !expanded[job.id]">
          {{ expanded[job.id] ? 'Hide log' : 'Show log' }}
        </button>
      </div>

      <pre v-if="expanded[job.id]" class="ffmpeg-log">{{ (jobsStore.logs[job.id] ?? []).join('\n') }}</pre>
    </div>
  </div>
</template>

<style scoped>
.history { margin-top: 20px; }
.history-header-row {
  display: flex; align-items: center; justify-content: space-between; margin-bottom: 10px;
}
.clear-btn { font-size: 12px; padding: 3px 10px; color: var(--muted); }
.history-card {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 12px 16px;
  margin-bottom: 8px;
}
.history-card--failed { border-color: rgba(239, 68, 68, 0.3); }
.history-card--completed { border-color: rgba(34, 197, 94, 0.2); }

.history-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}
.history-name {
  font-size: 13px;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 75%;
}

.status-badge {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 4px;
  font-weight: 500;
  text-transform: capitalize;
}
.badge--completed { background: rgba(34, 197, 94, 0.15); color: var(--success); }
.badge--failed { background: rgba(239, 68, 68, 0.15); color: var(--danger); }
.badge--cancelled { background: var(--elevated); color: var(--muted); }

.output-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 5px 8px;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: background 0.15s;
  margin-bottom: 2px;
}
.output-row:hover { background: var(--elevated); }
.output-icon { font-size: 13px; flex-shrink: 0; }
.output-name {
  font-size: 12px;
  color: var(--muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1;
}
.open-hint { font-size: 12px; color: var(--accent); flex-shrink: 0; }

.error-msg {
  font-size: 12px;
  color: var(--danger);
  background: rgba(239, 68, 68, 0.08);
  padding: 6px 10px;
  border-radius: var(--radius-sm);
  margin-bottom: 4px;
}

.history-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: 6px;
}
.history-time { font-size: 11px; color: var(--muted); }
.log-toggle {
  font-size: 11px;
  color: var(--muted);
  background: none;
  border: none;
  cursor: pointer;
  padding: 0;
  text-decoration: underline;
}
.log-toggle:hover { color: var(--text); }

.ffmpeg-log {
  margin-top: 8px;
  padding: 8px 10px;
  background: var(--elevated);
  border-radius: var(--radius-sm);
  font-size: 10px;
  line-height: 1.5;
  color: var(--muted);
  max-height: 160px;
  overflow-y: auto;
  white-space: pre-wrap;
  word-break: break-all;
}
</style>
