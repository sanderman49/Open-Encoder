<script setup lang="ts">
import { computed } from 'vue'
import { usePresetsStore } from '@/stores/presets'
import { useFileDialog } from '@/composables/useFileDialog'
import { FolderOpen } from 'lucide-vue-next'

const store = usePresetsStore()
const out = computed(() => store.currentConfig.output)
const hasAudioExport = computed(() => !!store.currentConfig.audioExport)
const { pickOutputDir } = useFileDialog()

async function browseVideoDir() {
  const dir = await pickOutputDir()
  if (dir) out.value.videoDir = dir
}

async function browseAudioDir() {
  const dir = await pickOutputDir()
  if (dir) out.value.audioDir = dir
}

const previewName = computed(() => {
  const prefix = out.value.filenamePrefix || ''
  const suffix = out.value.filenameSuffix || ''
  return `${prefix}video${suffix}.mp4`
})
</script>

<template>
  <div>
    <p class="section-title">Output</p>

    <!-- Output folder -->
    <div class="form-row">
      <label>Folder</label>
      <div class="path-row">
        <input v-model="out.videoDir" placeholder="Same as original video" class="path-input" />
        <button class="btn btn-ghost icon-btn" @click="browseVideoDir" title="Browse">
          <FolderOpen :size="14" />
        </button>
      </div>
    </div>

    <!-- Date subfolder -->
    <div class="form-row">
      <label>Date subfolder</label>
      <label class="toggle">
        <input v-model="out.createDateFolder" type="checkbox" />
        <span class="toggle-track" />
      </label>
    </div>
    <p v-if="out.createDateFolder" class="hint">
      Files saved inside a <code>YYYY-MM-DD</code> folder e.g. <code>2026-06-02/</code>
    </p>

    <!-- Filename prefix / suffix -->
    <div class="form-row">
      <label>Prefix</label>
      <input v-model="out.filenamePrefix" placeholder="e.g. export_" class="path-input" />
    </div>

    <div class="form-row">
      <label>Suffix</label>
      <input v-model="out.filenameSuffix" placeholder="e.g. _final" class="path-input" />
    </div>

    <p class="filename-preview">
      Output name: <code>{{ previewName }}</code>
    </p>

    <!-- Audio sub-section -->
    <template v-if="hasAudioExport">
      <p class="audio-section-title">AUDIO OUTPUT</p>

      <div class="form-row">
        <label>Folder</label>
        <div class="path-row">
          <input
            v-model="out.audioDir"
            :placeholder="out.audioDirRelative ? 'e.g. audio or ../exports' : 'Same as video'"
            class="path-input"
          />
          <button
            v-if="!out.audioDirRelative"
            class="btn btn-ghost icon-btn"
            @click="browseAudioDir"
            title="Browse"
          ><FolderOpen :size="14" /></button>
        </div>
      </div>

      <div class="form-row">
        <label>Relative to output folder</label>
        <label class="toggle">
          <input v-model="out.audioDirRelative" type="checkbox" />
          <span class="toggle-track" />
        </label>
      </div>
      <p v-if="out.audioDirRelative" class="hint">
        Folder is relative to the video output folder. Created automatically if missing.
      </p>
    </template>
  </div>
</template>

<style scoped>
.path-row {
  display: flex;
  align-items: center;
  gap: 6px;
  flex: 1;
  min-width: 0;
}
.path-input { flex: 1; min-width: 0; }
.icon-btn { padding: 6px 8px; flex-shrink: 0; }

.audio-section-title {
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.08em;
  color: var(--muted);
  margin: 20px 0 12px;
}

.hint {
  color: var(--muted);
  font-size: 12px;
  margin-top: -8px;
  margin-bottom: 14px;
}
.hint code {
  color: var(--text);
  font-size: 11px;
  font-family: monospace;
}

.filename-preview {
  font-size: 12px;
  color: var(--muted);
  margin-top: -4px;
  margin-bottom: 14px;
}
.filename-preview code {
  color: var(--text);
  font-family: monospace;
  font-size: 11px;
}
</style>
