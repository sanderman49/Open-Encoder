<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
const dots = ref(1)
let timer: ReturnType<typeof setInterval>
onMounted(() => { timer = setInterval(() => { dots.value = dots.value === 3 ? 1 : dots.value + 1 }, 500) })
onUnmounted(() => clearInterval(timer))
</script>
<template><span class="dots">{{ '.'.repeat(dots) }}</span></template>
<style scoped>
.dots { display: inline-block; min-width: 1.5em; text-align: left; }
</style>
