<script setup lang="ts">
import { ref } from 'vue'
import { usePresetsStore } from '@/stores/presets'
import { Settings, Trash2 } from 'lucide-vue-next'

const emit = defineEmits<{ openSettings: [] }>()

const store = usePresetsStore()
const saving = ref(false)
const newName = ref('')
const showSaveInput = ref(false)

function onPresetChange(e: Event) {
  const id = (e.target as HTMLSelectElement).value
  const preset = store.allPresets.find(p => p.id === id)
  if (preset) store.applyPreset(preset)
}

async function savePreset() {
  const name = newName.value.trim()
  if (!name) return
  await store.saveCurrentAsPreset(name)
  showSaveInput.value = false
  newName.value = ''
}

async function deleteActive() {
  const id = store.activePresetId
  if (id.startsWith('builtin-')) return
  await store.deletePreset(id)
}

const isBuiltin = () => store.activePresetId.startsWith('builtin-')
</script>

<template>
  <header class="preset-bar">
    <div class="preset-bar__left">
      <label class="preset-label">Preset</label>
      <select :value="store.activePresetId" @change="onPresetChange" class="preset-select">
        <optgroup label="Built-in">
          <option
            v-for="p in store.allPresets.filter(p => p.id.startsWith('builtin-'))"
            :key="p.id"
            :value="p.id"
          >{{ p.name }}</option>
        </optgroup>
        <optgroup v-if="store.userPresets.length" label="Saved">
          <option v-for="p in store.userPresets" :key="p.id" :value="p.id">{{ p.name }}</option>
        </optgroup>
      </select>
    </div>

    <div class="preset-bar__right">
      <template v-if="showSaveInput">
        <input
          v-model="newName"
          placeholder="Preset name…"
          class="name-input"
          @keyup.enter="savePreset"
          @keyup.escape="showSaveInput = false"
          autofocus
        />
        <button class="btn btn-primary" @click="savePreset">Save</button>
        <button class="btn btn-ghost" @click="showSaveInput = false">Cancel</button>
      </template>
      <template v-else>
        <button class="btn btn-ghost" @click="showSaveInput = true" title="Save current settings as preset">
          Save as…
        </button>
        <button
          v-if="!isBuiltin()"
          class="btn btn-ghost btn-del"
          @click="deleteActive"
          title="Delete this preset"
        ><Trash2 :size="14" /></button>
        <button class="btn btn-ghost icon-btn" @click="emit('openSettings')" title="Settings"><Settings :size="15" /></button>
      </template>
    </div>
  </header>
</template>

<style scoped>
.preset-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 20px;
  background: var(--surface);
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.preset-bar__left { display: flex; align-items: center; gap: 10px; }
.preset-bar__right { display: flex; align-items: center; gap: 8px; }

.preset-label { font-size: 12px; color: var(--muted); white-space: nowrap; }

.preset-select { min-width: 180px; }

.name-input {
  width: 180px;
  padding: 6px 10px;
}

.icon-btn { font-size: 16px; padding: 6px 10px; }
.btn-del { color: var(--danger); }
.btn-del:hover { border-color: var(--danger); }
</style>
