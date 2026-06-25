<script setup lang="ts">
import { ref } from 'vue'
import { useJobsStore } from '@/stores/jobs'
import type { Job } from '@/types/jobs'

defineProps<{ jobs: Job[] }>()
defineEmits<{ cancel: [id: string] }>()

const jobsStore = useJobsStore()
const expanded = ref<Record<string, boolean>>({})

function eta(s: number) {
  if (s <= 0) return ''
  if (s < 60) return `~${Math.round(s)}s`
  return `~${Math.round(s / 60)}m`
}

function basename(p: string) { return p.split(/[/\\]/).pop() ?? p }
</script>

<template>
  <div v-if="jobs.length" class="queue">
    <p class="section-title">Processing</p>
    <div v-for="job in jobs" :key="job.id" class="job-card">
      <div class="job-header">
        <div class="job-names">
          <span class="job-name">{{ basename(job.inputPath) }}</span>
          <span v-if="job.outputName && job.outputName !== basename(job.inputPath).replace(/\.[^.]+$/, '')" class="job-output-name">→ {{ job.outputName }}</span>
        </div>
        <button class="btn btn-ghost cancel-btn" @click="$emit('cancel', job.id)">Cancel</button>
      </div>

      <div class="phase-row">
        <span class="phase-label">Video</span>
        <div class="bar-wrap">
          <div class="bar" :style="{ width: job.videoPercent + '%' }" />
        </div>
        <span class="pct">{{ Math.round(job.videoPercent) }}%</span>
      </div>

      <div v-if="job.audioOutput !== undefined || job.activePhase === 'audio'" class="phase-row">
        <span class="phase-label">Audio</span>
        <div class="bar-wrap">
          <div class="bar bar--audio" :style="{ width: job.audioPercent + '%' }" />
        </div>
        <span class="pct">{{ Math.round(job.audioPercent) }}%</span>
      </div>

      <div class="job-meta">
        <span v-if="job.speed !== 'N/A'" class="speed">{{ job.speed }}</span>
        <span v-if="job.etaSeconds > 0" class="eta">{{ eta(job.etaSeconds) }}</span>
        <button class="log-toggle" @click="expanded[job.id] = !expanded[job.id]">
          {{ expanded[job.id] ? 'Hide log' : 'Show log' }}
        </button>
      </div>

      <pre v-if="expanded[job.id]" class="ffmpeg-log">{{ (jobsStore.logs[job.id] ?? []).join('\n') }}</pre>
    </div>
  </div>
</template>

<style scoped>
.queue { margin-top: 20px; }
.job-card {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 14px 16px;
  margin-bottom: 10px;
}
.job-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 10px;
}
.job-names {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
  max-width: 70%;
}
.job-name {
  font-size: 13px;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.job-output-name {
  font-size: 11px;
  color: var(--muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.cancel-btn { font-size: 12px; padding: 4px 10px; color: var(--danger); border-color: var(--danger); }
.cancel-btn:hover { background: rgba(239, 68, 68, 0.1); }

.phase-row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 6px;
}
.phase-label { font-size: 11px; color: var(--muted); width: 36px; flex-shrink: 0; }
.bar-wrap {
  flex: 1;
  height: 6px;
  background: var(--elevated);
  border-radius: 3px;
  overflow: hidden;
}
.bar {
  height: 100%;
  background: var(--accent);
  border-radius: 3px;
  transition: width 0.4s ease;
}
.bar--audio { background: var(--success); }
.pct { font-size: 11px; color: var(--muted); width: 32px; text-align: right; flex-shrink: 0; }

.job-meta {
  display: flex;
  gap: 12px;
  margin-top: 6px;
}
.speed, .eta { font-size: 11px; color: var(--muted); }
.log-toggle {
  margin-left: auto;
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
