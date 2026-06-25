<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { usePresetsStore } from '@/stores/presets'
import { DEFAULT_AUDIO_EXPORT_CONFIG } from '@/types/preset'
import type { AudioExportConfig, AudioFormat, SampleRate, BitDepth } from '@/types/preset'
import { useTemplateVars } from '@/composables/useTemplateVars'
import VarInfoButton from './VarInfoButton.vue'
import ToggleSwitch from './ToggleSwitch.vue'

const store = usePresetsStore()
const cfg = computed(() => store.currentConfig)
const out = computed(() => store.currentConfig.output)
const { expand } = useTemplateVars()

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

// Local draft keeps the controls visible (greyed) while export is off. When on,
// it IS the live config object so edits flow straight through.
const draft = ref<AudioExportConfig>(cfg.value.audioExport ?? { ...DEFAULT_AUDIO_EXPORT_CONFIG })
const enabled = computed(() => !!cfg.value.audioExport)

// Re-bind the draft to the live object when the active config changes (preset switch).
watch(() => cfg.value.audioExport, (a) => {
  if (a) draft.value = a
})

const isLossy = computed(() => FORMATS.find(f => f.value === draft.value.format)?.lossy ?? true)
const showBitDepth = computed(() => !isLossy.value)

function setExport(on: boolean) {
  if (on) {
    cfg.value.audioExport = draft.value
  } else {
    cfg.value.audioExport = null
    // At least one of video/audio must stay enabled.
    if (!cfg.value.video.videoEnabled) cfg.value.video.videoEnabled = true
  }
}

function onFormatChange(e: Event) {
  const fmt = (e.target as HTMLSelectElement).value as AudioFormat
  draft.value.format = fmt
  const lossy = FORMATS.find(f => f.value === fmt)?.lossy ?? true
  if (lossy) {
    draft.value.bitrate = '320k'
    delete draft.value.bitDepth
  } else {
    delete draft.value.bitrate
    draft.value.bitDepth = fmt === 'flac' ? 24 : 16
  }
}

// ── Output: audio title (inherits video title) + subfolder ──
const inheritedVideoStem = computed(() =>
  out.value.nameOverride ? expand(out.value.nameOverride) : 'Inherits video title',
)
const audioTitlePreview = computed(() =>
  out.value.audioNameOverride ? expand(out.value.audioNameOverride) : null,
)

const lastAudioDir = ref(out.value.audioDir || 'podcast')
const audioSubdirEnabled = ref(out.value.audioDir !== '')
watch(audioSubdirEnabled, (on) => {
  if (on) {
    out.value.audioDir = lastAudioDir.value
  } else {
    lastAudioDir.value = out.value.audioDir || lastAudioDir.value
    out.value.audioDir = ''
  }
})
</script>

<template>
  <div>
    <p class="section-title">Audio Export</p>

    <div class="form-row">
      <span class="toggle-label">
        <ToggleSwitch :model-value="enabled" @update:model-value="setExport" />
        Export audio file
      </span>
    </div>

    <div class="section-body" :class="{ disabled: !enabled }">
      <div class="form-row">
        <label>Format</label>
        <select :value="draft.format" @change="onFormatChange">
          <option v-for="f in FORMATS" :key="f.value" :value="f.value">{{ f.label }}</option>
        </select>
      </div>

      <div v-if="isLossy" class="form-row">
        <label>Bitrate</label>
        <select v-model="draft.bitrate">
          <option v-for="b in BITRATES" :key="b" :value="b">{{ b }}</option>
        </select>
      </div>

      <div v-if="showBitDepth" class="form-row">
        <label>Bit depth</label>
        <select v-model.number="draft.bitDepth">
          <option v-for="d in BIT_DEPTHS" :key="d" :value="d">{{ d }}-bit</option>
        </select>
      </div>

      <div class="form-row">
        <label>Sample rate</label>
        <select v-model.number="draft.sampleRate">
          <option v-for="r in SAMPLE_RATES" :key="r" :value="r">{{ r }} Hz</option>
        </select>
      </div>

      <div class="form-row">
        <label>Channels</label>
        <select v-model.number="draft.channels">
          <option :value="1">Mono</option>
          <option :value="2">Stereo</option>
        </select>
      </div>

      <!-- ── Output ── -->
      <p class="sub-title">Output</p>

      <div class="form-row">
        <label>Audio title</label>
        <div class="field-with-info">
          <input v-model="out.audioNameOverride" :placeholder="inheritedVideoStem" class="var-field" />
          <VarInfoButton />
        </div>
      </div>
      <p v-if="audioTitlePreview" class="filename-preview">→ <code>{{ audioTitlePreview }}.{{ draft.format }}</code></p>

      <div class="form-row">
        <span class="toggle-label">
          <ToggleSwitch v-model="audioSubdirEnabled" /> Audio subfolder
        </span>
        <div class="field-with-info">
          <input
            v-model="out.audioDir"
            placeholder="podcast"
            class="var-field subdir-input"
            :disabled="!audioSubdirEnabled"
            :class="{ disabled: !audioSubdirEnabled }"
          />
          <VarInfoButton />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.section-body {
  transition: opacity 0.1s ease;
}
.section-body.disabled {
  opacity: 0.4;
  pointer-events: none;
}

.sub-title {
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--muted);
  margin: 22px 0 12px;
}

.field-with-info {
  display: flex;
  align-items: center;
  gap: 6px;
  flex: 1;
  min-width: 0;
  justify-content: flex-end;
}

.filename-preview {
  font-size: 12px;
  color: var(--muted);
  margin-top: -12px;
  margin-bottom: 18px;
}
.filename-preview code { color: var(--text); font-family: monospace; font-size: 11px; word-break: break-all; }
</style>
