<script setup lang="ts">
import { computed } from 'vue'
import { usePresetsStore } from '@/stores/presets'
import type { DeinterlaceAlgo } from '@/types/preset'

const store = usePresetsStore()
const di = computed(() => store.currentConfig.video.deinterlace)
const isCopy = computed(() => store.currentConfig.video.codec === 'copy')

const ALGOS: { value: DeinterlaceAlgo; label: string; desc: string }[] = [
  { value: 'bwdif', label: 'bwdif', desc: 'Best quality, motion-adaptive' },
  { value: 'yadif', label: 'yadif', desc: 'Fast, widely compatible' },
  { value: 'estdif', label: 'estdif', desc: 'Edge-slope tracing' },
]
</script>

<template>
  <div>
    <p class="section-title">Deinterlace</p>

    <div v-if="isCopy" class="notice">
      Deinterlace requires re-encoding. Select a codec above.
    </div>

    <template v-else>
      <div class="form-row">
        <label>Enable deinterlace</label>
        <label class="toggle">
          <input v-model="di.enabled" type="checkbox" />
          <span class="toggle-track" />
        </label>
      </div>

      <template v-if="di.enabled">
        <div class="form-row">
          <label>Auto-detect</label>
          <label class="toggle">
            <input v-model="di.autoDetect" type="checkbox" />
            <span class="toggle-track" />
          </label>
        </div>
        <p v-if="di.autoDetect" class="hint">
          Filter skipped if source is progressive
        </p>

        <div class="form-row">
          <label>Algorithm</label>
          <select v-model="di.algorithm">
            <option v-for="a in ALGOS" :key="a.value" :value="a.value">
              {{ a.label }} — {{ a.desc }}
            </option>
          </select>
        </div>
      </template>
    </template>
  </div>
</template>

<style scoped>
.notice {
  color: var(--muted);
  font-size: 12px;
  padding: 8px 12px;
  background: var(--elevated);
  border-radius: var(--radius-sm);
  border: 1px solid var(--border);
}
.hint { color: var(--muted); font-size: 12px; margin-top: -8px; margin-bottom: 14px; }
</style>
