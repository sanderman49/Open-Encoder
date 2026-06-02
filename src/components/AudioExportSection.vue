<script setup lang="ts">
import { computed } from 'vue'
import { usePresetsStore } from '@/stores/presets'
import type { AudioFormat, SampleRate, BitDepth } from '@/types/preset'

const store = usePresetsStore()
const cfg = computed(() => store.currentConfig)

const FORMATS: { value: AudioFormat; label: string; lossy: boolean }[] = [
  { value: 'mp3',  label: 'MP3',  lossy: true  },
  { value: 'm4a',  label: 'M4A (AAC)', lossy: true  },
  { value: 'ogg',  label: 'OGG (Vorbis)', lossy: true  },
  { value: 'opus', label: 'Opus', lossy: true  },
  { value: 'flac', label: 'FLAC', lossy: false },
  { value: 'wav',  label: 'WAV',  lossy: false },
  { value: 'aiff', label: 'AIFF', lossy: false },
]

const BITRATES = ['64k', '96k', '128k', '192k', '256k', '320k']
const SAMPLE_RATES: SampleRate[] = [22050, 44100, 48000, 96000, 192000]
const BIT_DEPTHS: BitDepth[] = [16, 24, 32]

const isLossy = computed(() => {
  if (!cfg.value.audioExport) return true
  return FORMATS.find(f => f.value === cfg.value.audioExport?.format)?.lossy ?? true
})

const showBitDepth = computed(() =>
  cfg.value.audioExport && !isLossy.value && cfg.value.audioExport.format !== 'flac'
    ? true
    : cfg.value.audioExport?.format === 'flac'
)

function toggleExport(e: Event) {
  const on = (e.target as HTMLInputElement).checked
  if (on) {
    cfg.value.audioExport = {
      format: 'mp3',
      bitrate: '320k',
      sampleRate: 48000,
      channels: 2,
    }
  } else {
    cfg.value.audioExport = null
  }
}

function onFormatChange(e: Event) {
  if (!cfg.value.audioExport) return
  const fmt = (e.target as HTMLSelectElement).value as AudioFormat
  cfg.value.audioExport.format = fmt
  const lossy = FORMATS.find(f => f.value === fmt)?.lossy ?? true
  if (lossy) {
    cfg.value.audioExport.bitrate = '320k'
    delete cfg.value.audioExport.bitDepth
  } else {
    delete cfg.value.audioExport.bitrate
    cfg.value.audioExport.bitDepth = fmt === 'flac' ? 24 : 16
  }
}
</script>

<template>
  <div>
    <p class="section-title">Audio Export</p>

    <div class="form-row">
      <label>Export audio file</label>
      <label class="toggle">
        <input type="checkbox" :checked="!!cfg.audioExport" @change="toggleExport" />
        <span class="toggle-track" />
      </label>
    </div>

    <template v-if="cfg.audioExport">
      <div class="form-row">
        <label>Format</label>
        <select :value="cfg.audioExport.format" @change="onFormatChange">
          <option v-for="f in FORMATS" :key="f.value" :value="f.value">{{ f.label }}</option>
        </select>
      </div>

      <div v-if="isLossy" class="form-row">
        <label>Bitrate</label>
        <select v-model="cfg.audioExport.bitrate">
          <option v-for="b in BITRATES" :key="b" :value="b">{{ b }}</option>
        </select>
      </div>

      <div v-if="showBitDepth" class="form-row">
        <label>Bit depth</label>
        <select v-model.number="cfg.audioExport.bitDepth">
          <option v-for="d in BIT_DEPTHS" :key="d" :value="d">{{ d }}-bit</option>
        </select>
      </div>

      <div class="form-row">
        <label>Sample rate</label>
        <select v-model.number="cfg.audioExport.sampleRate">
          <option v-for="r in SAMPLE_RATES" :key="r" :value="r">{{ r }} Hz</option>
        </select>
      </div>

      <div class="form-row">
        <label>Channels</label>
        <select v-model.number="cfg.audioExport.channels">
          <option :value="1">Mono</option>
          <option :value="2">Stereo</option>
        </select>
      </div>
    </template>
  </div>
</template>
