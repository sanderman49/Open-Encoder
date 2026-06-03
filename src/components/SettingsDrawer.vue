<script setup lang="ts">
import VideoSection from './VideoSection.vue'
import DeinterlaceSection from './DeinterlaceSection.vue'
import AudioExportSection from './AudioExportSection.vue'
import OutputSection from './OutputSection.vue'

const props = defineProps<{ open: boolean }>()
const emit = defineEmits<{ 'update:open': [value: boolean] }>()

function close() { emit('update:open', false) }
</script>

<template>
  <Teleport to="body">
    <Transition name="overlay">
      <div v-if="open" class="overlay" @click.self="close" />
    </Transition>
    <Transition name="drawer">
      <aside v-if="open" class="drawer">
        <div class="drawer__header">
          <span class="drawer__title">Settings</span>
          <button class="close-btn" @click="close">✕</button>
        </div>
        <div class="drawer__body">
          <VideoSection />
          <div class="divider" />
          <DeinterlaceSection />
          <div class="divider" />
          <AudioExportSection />
          <div class="divider" />
          <OutputSection />
        </div>
      </aside>
    </Transition>
  </Teleport>
</template>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  z-index: 100;
}
.drawer {
  position: fixed;
  top: 0;
  right: 0;
  bottom: 0;
  width: 340px;
  background: var(--surface);
  border-left: 1px solid var(--border);
  z-index: 101;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
.drawer__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}
.drawer__title { font-weight: 600; font-size: 15px; }
.close-btn {
  color: var(--muted);
  font-size: 14px;
  padding: 4px 8px;
  border-radius: 4px;
  transition: color 0.15s;
}
.close-btn:hover { color: var(--text); }

.drawer__body {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
}

/* Transitions */
.overlay-enter-active, .overlay-leave-active { transition: opacity 0.2s; }
.overlay-enter-from, .overlay-leave-to { opacity: 0; }

.drawer-enter-active, .drawer-leave-active { transition: transform 0.25s ease; }
.drawer-enter-from, .drawer-leave-to { transform: translateX(100%); }
</style>
