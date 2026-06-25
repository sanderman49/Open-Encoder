<script setup lang="ts">
import { Info } from 'lucide-vue-next'
</script>

<template>
  <span class="infotip">
    <Info :size="13" class="infotip__icon" />
    <span class="infotip__bubble"><slot /></span>
  </span>
</template>

<style scoped>
.infotip {
  position: relative;
  display: inline-flex;
  align-items: center;
  vertical-align: middle;
}
.infotip__icon {
  color: var(--muted);
  cursor: help;
  transition: color 0.15s;
}
.infotip:hover .infotip__icon { color: var(--accent); }

.infotip__bubble {
  position: absolute;
  top: calc(100% + 6px);
  right: 0;                 /* right-anchored so it grows left, never off the right edge */
  z-index: 60;
  width: max-content;
  max-width: 220px;
  padding: 7px 10px;
  background: var(--elevated);
  border: 1px solid var(--border);
  border-radius: 6px;
  box-shadow: 0 6px 18px rgba(0, 0, 0, 0.45);
  font-size: 11px;
  line-height: 1.4;
  color: var(--text);
  white-space: normal;
  opacity: 0;
  visibility: hidden;
  transition: opacity 0.1s ease;
  pointer-events: none;
}
.infotip:hover .infotip__bubble {
  opacity: 1;
  visibility: visible;
}
</style>
