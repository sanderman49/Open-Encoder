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

const now = new Date()
const pad = (n: number) => String(n).padStart(2, '0')
const dateStr     = `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}`
const timeStr     = `${pad(now.getHours())}-${pad(now.getMinutes())}-${pad(now.getSeconds())}`
const datetimeStr = `${dateStr}_${timeStr}`

const previewStem = computed(() => {
  if (!out.value.nameOverride) return null
  return out.value.nameOverride
    .replace(/\$DATETIME/g, datetimeStr)
    .replace(/\$DATE/g, dateStr)
    .replace(/\$TIME/g, timeStr)
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

    <!-- Name override -->
    <div class="form-row">
      <label>Name</label>
      <input
        v-model="out.nameOverride"
        placeholder="Uses title from main screen"
        class="path-input"
      />
    </div>
    <p class="hint">Leave empty to use the editable title. Variables: <code>$DATE</code> <code>$TIME</code> <code>$DATETIME</code></p>
    <p v-if="previewStem" class="filename-preview">
      Output name: <code>{{ previewStem }}.mp4</code>
    </p>

    <!-- Audio sub-section -->
    <template v-if="hasAudioExport">
      <p class="audio-section-title">AUDIO OUTPUT</p>

      <div class="form-row">
        <label>Subfolder</label>
        <input
          v-model="out.audioDir"
          placeholder="Same as video (e.g. audio)"
          class="path-input"
        />
      </div>
      <p class="hint">Relative to video output folder. Created automatically if missing.</p>
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
  margin-top: -8px;
  margin-bottom: 14px;
}
.filename-preview code {
  color: var(--text);
  font-family: monospace;
  font-size: 11px;
}
</style>
