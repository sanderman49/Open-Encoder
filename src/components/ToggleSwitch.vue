<script setup lang="ts">
defineProps<{ modelValue: boolean }>()
const emit = defineEmits<{ 'update:modelValue': [value: boolean] }>()
</script>

<template>
  <button
    type="button"
    class="switch"
    :class="{ 'switch--on': modelValue }"
    role="switch"
    :aria-checked="modelValue"
    @click="emit('update:modelValue', !modelValue)"
  >
    <span class="switch__thumb" />
  </button>
</template>

<style scoped>
.switch {
  position: relative;
  width: 28px;
  height: 16px;
  border-radius: 8px;
  background: var(--border);
  padding: 0;
  flex-shrink: 0;
  transition: background 0.2s;
}
.switch--on { background: var(--accent); }

.switch__thumb {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: #fff;
  transition: transform 0.2s;
}
.switch--on .switch__thumb { transform: translateX(12px); }
</style>
