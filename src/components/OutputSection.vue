<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { usePresetsStore } from '@/stores/presets'
import { useFileDialog } from '@/composables/useFileDialog'
import { useTemplateVars } from '@/composables/useTemplateVars'
import VarInfoButton from './VarInfoButton.vue'
import ToggleSwitch from './ToggleSwitch.vue'
import { FolderOpen } from 'lucide-vue-next'

const store = usePresetsStore()
const out = computed(() => store.currentConfig.output)
const { expand, hasVars } = useTemplateVars()
const { pickOutputDir } = useFileDialog()

async function browseVideoDir() {
  const dir = await pickOutputDir()
  if (dir) out.value.videoDir = dir
}

const previewFolder = computed(() =>
  hasVars(out.value.videoDir) ? expand(out.value.videoDir) : null,
)

const lastExportDir = ref(out.value.exportDir || 'output')
const exportDirEnabled = ref(out.value.exportDir !== '')
watch(exportDirEnabled, (on) => {
  if (on) {
    out.value.exportDir = lastExportDir.value
  } else {
    lastExportDir.value = out.value.exportDir || lastExportDir.value
    out.value.exportDir = ''
  }
})

const exportDirPreview = computed(() =>
  exportDirEnabled.value && hasVars(out.value.exportDir) ? expand(out.value.exportDir) : null,
)
</script>

<template>
  <div>
    <p class="section-title">Path</p>

    <div class="form-row">
      <label>Folder</label>
      <div class="path-row">
        <input v-model="out.videoDir" placeholder="Same as original video" class="var-field path-input" />
        <button class="btn btn-ghost icon-btn" @click="browseVideoDir" title="Browse">
          <FolderOpen :size="14" />
        </button>
        <VarInfoButton />
      </div>
    </div>
    <p v-if="previewFolder" class="filename-preview">→ <code>{{ previewFolder }}</code></p>

    <div class="form-row">
      <span class="toggle-label">
        <ToggleSwitch v-model="exportDirEnabled" /> Export folder
      </span>
      <div class="field-with-info">
        <input
          v-model="out.exportDir"
          placeholder="output"
          class="var-field subdir-input"
          :disabled="!exportDirEnabled"
          :class="{ disabled: !exportDirEnabled }"
        />
        <VarInfoButton />
      </div>
    </div>
    <p v-if="exportDirPreview" class="filename-preview">→ <code>{{ exportDirPreview }}</code></p>
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

.field-with-info {
  display: flex;
  align-items: center;
  gap: 6px;
  flex: 1;
  min-width: 0;
  justify-content: flex-end;
}

.filename-preview {
  font-size: 12px;
  color: var(--muted);
  margin-top: -12px;
  margin-bottom: 18px;
}
.filename-preview code { color: var(--text); font-family: monospace; font-size: 11px; word-break: break-all; }
</style>
