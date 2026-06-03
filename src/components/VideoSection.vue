<script setup lang="ts">
import { computed, ref, onMounted } from 'vue'
import { platform } from '@tauri-apps/plugin-os'
import { usePresetsStore } from '@/stores/presets'
import { CODEC_CONTAINERS } from '@/types/preset'
import type { VideoCodec, Container, Resolution, EncodePreset, HwAccel } from '@/types/preset'

const store = usePresetsStore()
const v = computed(() => store.currentConfig.video)

const CODECS: { value: VideoCodec; label: string }[] = [
  { value: 'libx264', label: 'H.264 (libx264)' },
  { value: 'libx265', label: 'H.265 / HEVC (libx265)' },
  { value: 'libvp9',  label: 'VP9 (libvp9)' },
  { value: 'libsvtav1', label: 'AV1 (SVT-AV1)' },
  { value: 'copy',    label: 'Passthrough (copy)' },
]

const CONTAINERS: { value: Container; label: string }[] = [
  { value: 'mp4',  label: 'MP4' },
  { value: 'mkv',  label: 'MKV' },
  { value: 'webm', label: 'WebM' },
  { value: 'mov',  label: 'MOV' },
]

const RESOLUTIONS: { value: Resolution; label: string }[] = [
  { value: 'source', label: 'Original (unchanged)' },
  { value: '480p',   label: '480p' },
  { value: '720p',   label: '720p HD' },
  { value: '1080p',  label: '1080p Full HD' },
  { value: '1440p',  label: '1440p QHD' },
  { value: '2160p',  label: '2160p 4K' },
  { value: 'custom', label: 'Custom…' },
]

const PRESETS: EncodePreset[] = [
  'ultrafast','superfast','veryfast','faster','fast','medium','slow','slower','veryslow',
]

const ALL_HW_ACCELS: { value: HwAccel; label: string; os: string[] }[] = [
  { value: 'nvenc',        label: 'NVENC (Nvidia)',           os: ['windows', 'linux'] },
  { value: 'amf',          label: 'AMF (AMD)',                os: ['windows'] },
  { value: 'qsv',          label: 'Quick Sync (Intel)',       os: ['windows', 'linux'] },
  { value: 'videotoolbox', label: 'VideoToolbox (Apple)',     os: ['macos'] },
  { value: 'vaapi',        label: 'VAAPI (AMD/Intel Linux)',  os: ['linux'] },
]

const currentOs = ref('linux')
const HW_ACCELS = computed(() => ALL_HW_ACCELS.filter(h => h.os.includes(currentOs.value)))

onMounted(async () => {
  currentOs.value = await platform()
  if (v.value.hwAccel !== 'none' && !HW_ACCELS.value.find(h => h.value === v.value.hwAccel)) {
    v.value.hwAccel = 'none'
  }
})

const validContainers = computed(() => CODEC_CONTAINERS[v.value.codec] ?? [])
const showCrf = computed(() => v.value.codec !== 'copy')
const showEncodePreset = computed(() => !['libvp9', 'libsvtav1', 'copy'].includes(v.value.codec))
const showCustomRes = computed(() => v.value.resolution === 'custom')
const showResolution = computed(() => v.value.codec !== 'copy')
const showHwAccel = computed(() => ['libx264', 'libx265'].includes(v.value.codec))

function onCodecChange(e: Event) {
  const codec = (e.target as HTMLSelectElement).value as VideoCodec
  v.value.codec = codec
  const valid = CODEC_CONTAINERS[codec]
  if (!valid.includes(v.value.container)) v.value.container = valid[0]
  if (codec === 'copy') { v.value.resolution = 'source'; v.value.encodePreset = null }
  if (['libvp9', 'libsvtav1', 'copy'].includes(codec)) {
    v.value.encodePreset = null
    v.value.hwAccel = 'none'
  }
}
</script>

<template>
  <div>
    <p class="section-title">Video</p>

    <div class="form-row">
      <label>Codec</label>
      <select :value="v.codec" @change="onCodecChange">
        <option v-for="c in CODECS" :key="c.value" :value="c.value">{{ c.label }}</option>
      </select>
    </div>

    <div class="form-row">
      <label>Container</label>
      <select v-model="v.container">
        <option
          v-for="c in CONTAINERS"
          :key="c.value"
          :value="c.value"
          :disabled="!validContainers.includes(c.value)"
        >{{ c.label }}</option>
      </select>
    </div>

    <div v-if="showResolution" class="form-row">
      <label>Resolution</label>
      <select v-model="v.resolution">
        <option v-for="r in RESOLUTIONS" :key="r.value" :value="r.value">{{ r.label }}</option>
      </select>
    </div>

    <div v-if="showCustomRes" class="custom-res">
      <input v-model.number="v.customWidth" type="number" placeholder="Width" min="1" />
      <span class="sep">×</span>
      <input v-model.number="v.customHeight" type="number" placeholder="Height" min="1" />
    </div>

    <div v-if="showCrf" class="form-row crf-row">
      <label>Quality (CRF {{ v.crf }})</label>
      <div class="range-wrap">
        <span class="range-label">Best</span>
        <input v-model.number="v.crf" type="range" min="0" max="51" step="1" />
        <span class="range-label">Worst</span>
      </div>
    </div>

    <div v-if="showEncodePreset" class="form-row">
      <label>Speed</label>
      <select v-model="v.encodePreset">
        <option v-for="p in PRESETS" :key="p" :value="p">{{ p }}</option>
      </select>
    </div>

    <div v-if="showHwAccel" class="form-row">
      <label>Hardware encoder</label>
      <select v-model="v.hwAccel">
        <option v-for="h in HW_ACCELS" :key="h.value" :value="h.value">{{ h.label }}</option>
      </select>
    </div>

    <div v-if="v.hwAccel === 'vaapi'" class="form-row">
      <label>VAAPI device</label>
      <input v-model="v.vaapiDevice" placeholder="/dev/dri/renderD128" />
    </div>

  </div>
</template>

<style scoped>
.custom-res {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 14px;
}
.custom-res input { width: 90px; }
.sep { color: var(--muted); }

.crf-row { flex-direction: column; align-items: flex-start; }
.crf-row label { margin-bottom: 6px; }

.range-wrap {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
}
.range-wrap input { flex: 1; }
.range-label { font-size: 11px; color: var(--muted); white-space: nowrap; }
</style>
