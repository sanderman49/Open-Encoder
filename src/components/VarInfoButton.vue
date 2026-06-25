<script setup lang="ts">
import { ref, reactive, onUnmounted } from 'vue'
import { Info } from 'lucide-vue-next'
import { useTemplateVars } from '@/composables/useTemplateVars'

const { VARS } = useTemplateVars()

const root = ref<HTMLElement | null>(null)
const popover = ref<HTMLElement | null>(null)
const open = ref(false)

// Fixed-position style; the popover is teleported to <body> and centered within
// the main area (the region to the LEFT of the settings drawer) to use all the room.
const style = reactive<{ left: string; maxWidth: string }>({ left: '50%', maxWidth: 'none' })

function computeStyle() {
  const btn = root.value
  if (!btn) return
  const drawer = btn.closest('.drawer') as HTMLElement | null
  const drawerLeft = drawer ? drawer.getBoundingClientRect().left : window.innerWidth
  style.left = `${drawerLeft / 2}px`
  style.maxWidth = `${Math.max(drawerLeft - 32, 0)}px`
}

function onDocPointerDown(e: PointerEvent) {
  const target = e.target as HTMLElement
  // Keep open while interacting with this button, the popover, or ANY variable field.
  if (root.value?.contains(target)) return
  if (popover.value?.contains(target)) return
  if (target.closest('.var-field')) return
  close()
}

function openPopover() {
  if (open.value) return
  computeStyle()
  open.value = true
  document.addEventListener('pointerdown', onDocPointerDown, true)
  window.addEventListener('resize', computeStyle)
}

function close() {
  if (!open.value) return
  open.value = false
  document.removeEventListener('pointerdown', onDocPointerDown, true)
  window.removeEventListener('resize', computeStyle)
}

function toggle() {
  open.value ? close() : openPopover()
}

onUnmounted(close)
</script>

<template>
  <span ref="root" class="var-info">
    <button
      type="button"
      class="info-btn"
      :class="{ 'info-btn--on': open }"
      title="Show available variables"
      @click="toggle"
    >
      <Info :size="13" />
    </button>

    <Teleport to="body">
      <div
        v-if="open"
        ref="popover"
        class="var-popover"
        :style="{ left: style.left, maxWidth: style.maxWidth }"
      >
        <p class="var-popover__title">Variables</p>
        <div class="var-popover__list">
          <div v-for="item in VARS" :key="item.name" class="var-row">
            <code class="var-name">{{ item.name }}</code>
            <span class="var-desc">{{ item.desc }}</span>
            <code class="var-example">{{ item.example }}</code>
          </div>
        </div>
      </div>
    </Teleport>
  </span>
</template>

<style scoped>
.var-info {
  display: inline-flex;
  flex-shrink: 0;
}

.info-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 4px;
  border-radius: 4px;
  color: var(--muted);
  transition: color 0.15s, background 0.15s;
}
.info-btn:hover,
.info-btn--on { color: var(--accent); }
</style>

<style>
/* Unscoped: the popover is teleported to <body>, outside this component's tree. */
.var-popover {
  position: fixed;
  top: 50%;
  transform: translate(-50%, -50%);
  z-index: 200;
  width: 520px;
  max-height: calc(100vh - 32px);
  overflow-y: auto;
  background: var(--elevated);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 16px 18px;
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.55);
  animation: var-pop-in 0.1s ease;
}

/* Fade in on mount — runs every time the popover is (re)created. */
@keyframes var-pop-in {
  from { opacity: 0; }
  to { opacity: 1; }
}

.var-popover__title {
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: var(--muted);
  margin-bottom: 8px;
}

.var-popover__list { display: flex; flex-direction: column; }

.var-popover .var-row {
  display: grid;
  grid-template-columns: 92px 1fr auto;
  align-items: baseline;
  gap: 10px;
  padding: 6px 4px;
  font-size: 12px;
  border-bottom: 1px solid var(--border);
}
.var-popover .var-row:last-child { border-bottom: none; }

.var-popover .var-name {
  font-family: monospace;
  font-size: 11px;
  color: var(--accent);
}
.var-popover .var-desc { color: var(--muted); }
.var-popover .var-example {
  font-family: monospace;
  font-size: 11px;
  color: var(--text);
  opacity: 0.55;
  white-space: nowrap;
}
</style>
