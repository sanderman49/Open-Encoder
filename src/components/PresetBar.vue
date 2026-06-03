<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { usePresetsStore } from '@/stores/presets'
import { Settings, MoreHorizontal, Plus } from 'lucide-vue-next'

const emit = defineEmits<{ openSettings: [] }>()

const store = usePresetsStore()

const showNewInput = ref(false)
const newName = ref('')

const menuOpen = ref(false)
const menuMode = ref<'main' | 'save-as' | 'rename'>('main')
const renameValue = ref('')
const saveAsName = ref('')
const menuRef = ref<HTMLElement | null>(null)

// Delete confirmation overlay
const showDeleteConfirm = ref(false)
const deletePillStyle = ref<Record<string, string>>({})
const deletePresetName = ref('')

const isBuiltin = () => store.activePresetId.startsWith('builtin-')

function onPresetChange(e: Event) {
  const id = (e.target as HTMLSelectElement).value
  const preset = store.allPresets.find(p => p.id === id)
  if (preset) store.applyPreset(preset, true)
}

async function createPreset() {
  const name = newName.value.trim()
  if (!name) return
  await store.saveCurrentAsPreset(name)
  showNewInput.value = false
  newName.value = ''
}

async function saveActive() {
  const preset = store.activePreset
  if (!preset || isBuiltin()) return
  await store.saveCurrentAsPreset(preset.name)
}

function openSaveAs() {
  saveAsName.value = store.activePreset?.name ? `${store.activePreset.name} copy` : ''
  menuMode.value = 'save-as'
  menuOpen.value = true
}

function openMenu() {
  menuMode.value = 'main'
  menuOpen.value = true
}

function closeMenu() {
  menuOpen.value = false
  menuMode.value = 'main'
}

function startRename() {
  renameValue.value = store.activePreset?.name ?? ''
  menuMode.value = 'rename'
}

async function confirmRename() {
  if (!renameValue.value.trim()) return
  await store.renamePreset(store.activePresetId, renameValue.value)
  closeMenu()
}

async function confirmSaveAs() {
  if (!saveAsName.value.trim()) return
  await store.saveCurrentAsPreset(saveAsName.value.trim())
  closeMenu()
}

// Show delete popup with Cancel button under the cursor.
// Layout (L→R): filename | Delete | Cancel
// Cancel is rightmost, so anchor pill's right edge to cursor.
// Cancel btn: ~72px wide. Pill padding-right: 14px.
// Cancel center from pill right = 14 + 36 = 50px → pill right edge = clientX + 50
function requestDelete(e: MouseEvent) {
  deletePresetName.value = store.activePreset?.name ?? ''
  closeMenu()
  const pillRight = e.clientX + 50           // right edge of pill in viewport coords
  const rightFromEdge = Math.max(8, window.innerWidth - pillRight)
  const top = Math.max(8, Math.min(e.clientY - 26, window.innerHeight - 60))
  deletePillStyle.value = {
    right: `${rightFromEdge}px`,
    top:   `${top}px`,
  }
  showDeleteConfirm.value = true
}

async function confirmDelete() {
  await store.deletePreset(store.activePresetId)
  showDeleteConfirm.value = false
}

function cancelDelete() {
  showDeleteConfirm.value = false
}

function onClickOutside(e: MouseEvent) {
  if (menuRef.value && !menuRef.value.contains(e.target as Node)) closeMenu()
}

onMounted(() => document.addEventListener('mousedown', onClickOutside))
onUnmounted(() => document.removeEventListener('mousedown', onClickOutside))
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

      <template v-if="showNewInput">
        <input
          v-model="newName"
          placeholder="Preset name…"
          class="name-input"
          @keyup.enter="createPreset"
          @keyup.escape="showNewInput = false"
          autofocus
        />
        <button class="btn btn-primary" @click="createPreset">Create</button>
        <button class="btn btn-ghost" @click="showNewInput = false; newName = ''">Cancel</button>
      </template>
      <button v-else class="btn btn-ghost icon-btn" @click="showNewInput = true" title="New preset">
        <Plus :size="14" />
      </button>
    </div>

    <div class="preset-bar__right">
      <template v-if="isBuiltin()">
        <button class="btn btn-ghost" @click="openSaveAs">Save as…</button>
      </template>
      <template v-else>
        <button class="btn btn-ghost" @click="saveActive">Save</button>
        <button class="btn btn-ghost icon-btn" @click="openMenu" title="Preset actions">
          <MoreHorizontal :size="15" />
        </button>
      </template>

      <div class="menu-anchor" ref="menuRef">
        <div v-if="menuOpen" class="menu">
          <template v-if="menuMode === 'main'">
            <button class="menu-item" @click="saveAsName = (store.activePreset?.name ?? '') + ' copy'; menuMode = 'save-as'">Save as…</button>
            <button class="menu-item" @click="startRename">Rename</button>
            <div class="menu-divider" />
            <button class="menu-item menu-item--danger" @click="requestDelete($event)">Delete…</button>
          </template>

          <template v-else-if="menuMode === 'save-as'">
            <p class="menu-label">Save as new preset</p>
            <input v-model="saveAsName" class="menu-input" placeholder="Preset name…"
              @keyup.enter="confirmSaveAs" @keyup.escape="closeMenu" autofocus />
            <div class="menu-actions">
              <button class="btn btn-ghost menu-btn" @click="closeMenu">Cancel</button>
              <button class="btn btn-primary menu-btn" @click="confirmSaveAs">Save</button>
            </div>
          </template>

          <template v-else-if="menuMode === 'rename'">
            <p class="menu-label">Rename preset</p>
            <input v-model="renameValue" class="menu-input"
              @keyup.enter="confirmRename" @keyup.escape="closeMenu" autofocus />
            <div class="menu-actions">
              <button class="btn btn-ghost menu-btn" @click="closeMenu">Cancel</button>
              <button class="btn btn-primary menu-btn" @click="confirmRename">Rename</button>
            </div>
          </template>
        </div>
      </div>

      <!-- Save as popup for builtins (anchored same way) -->
      <div v-if="isBuiltin() && menuOpen" class="menu-anchor">
        <div class="menu">
          <p class="menu-label">Save as new preset</p>
          <input v-model="saveAsName" class="menu-input" placeholder="Preset name…"
            @keyup.enter="confirmSaveAs" @keyup.escape="closeMenu" autofocus />
          <div class="menu-actions">
            <button class="btn btn-ghost menu-btn" @click="closeMenu">Cancel</button>
            <button class="btn btn-primary menu-btn" @click="confirmSaveAs">Save</button>
          </div>
        </div>
      </div>

      <button class="btn btn-ghost icon-btn" @click="emit('openSettings')" title="Settings">
        <Settings :size="15" />
      </button>
    </div>
  </header>

  <!-- Delete confirmation overlay -->
  <Teleport to="body">
    <Transition name="fade">
      <div v-if="showDeleteConfirm" class="delete-overlay" @click.self="cancelDelete">
        <div class="delete-pill" :style="deletePillStyle">
          <span class="delete-pill__label">Delete "{{ deletePresetName }}"?</span>
          <button class="btn btn-danger delete-pill__confirm" @click="confirmDelete">Delete</button>
          <button class="btn btn-ghost delete-pill__cancel" @click="cancelDelete">Cancel</button>
        </div>
      </div>
    </Transition>
  </Teleport>
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

.preset-bar__left { display: flex; align-items: center; gap: 8px; flex: 1; min-width: 0; }
.preset-bar__right { display: flex; align-items: center; gap: 8px; flex-shrink: 0; }

.preset-label { font-size: 12px; color: var(--muted); white-space: nowrap; }
.preset-select { min-width: 160px; }
.name-input { width: 160px; padding: 6px 10px; }
.icon-btn { padding: 6px 8px; }

/* Dropdown */
.menu-anchor { position: relative; }

.menu {
  position: absolute;
  top: 6px;
  right: 0;
  background: var(--elevated);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  box-shadow: 0 8px 24px rgba(0,0,0,0.4);
  min-width: 180px;
  z-index: 200;
  padding: 6px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.menu-item {
  display: block;
  width: 100%;
  text-align: left;
  padding: 7px 10px;
  border-radius: var(--radius-sm);
  font-size: 13px;
  color: var(--text);
  transition: background 0.12s;
}
.menu-item:hover { background: var(--border); }
.menu-item--danger { color: var(--danger); }
.menu-item--danger:hover { background: rgba(239,68,68,0.12); }
.menu-divider { height: 1px; background: var(--border); margin: 4px 0; }
.menu-label { font-size: 12px; color: var(--muted); padding: 4px 6px 6px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.menu-input { width: 100%; margin-bottom: 6px; }
.menu-actions { display: flex; gap: 6px; justify-content: flex-end; }
.menu-btn { font-size: 12px; padding: 5px 10px; }

/* Delete overlay */
.delete-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.55);
  z-index: 500;
  backdrop-filter: blur(1px);
}

.delete-pill {
  position: fixed;
  display: flex;
  align-items: center;
  gap: 10px;
  background: var(--elevated);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 10px 14px;
  box-shadow: 0 8px 32px rgba(0,0,0,0.5);
  white-space: nowrap;
}

.delete-pill__label {
  font-size: 13px;
  color: var(--text);
  margin-right: 4px;
}

.delete-pill__cancel,
.delete-pill__confirm {
  font-size: 13px;
  padding: 5px 14px;
}

/* Transition */
.fade-enter-active, .fade-leave-active { transition: opacity 0.15s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
</style>
