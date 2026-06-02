<script setup lang="ts">
import { onMounted } from 'vue'
import { usePresetsStore } from '@/stores/presets'
import Home from '@/views/Home.vue'

const presetsStore = usePresetsStore()
onMounted(() => presetsStore.load())
</script>

<template>
  <Home />
</template>
