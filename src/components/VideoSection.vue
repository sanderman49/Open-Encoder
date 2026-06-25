<script setup lang="ts">
import { computed, ref, watch, onMounted } from 'vue'
import { platform } from '@tauri-apps/plugin-os'
import { invoke } from '@tauri-apps/api/core'
import { usePresetsStore } from '@/stores/presets'
import { CODEC_CONTAINERS, DEFAULT_AUDIO_EXPORT_CONFIG } from '@/types/preset'
import type { VideoCodec, Container, Resolution, EncodePreset, HwAccel, Framerate, DeinterlaceAlgo } from '@/types/preset'
import { useTemplateVars } from '@/composables/useTemplateVars'
import VarInfoButton from './VarInfoButton.vue'
import ToggleSwitch from './ToggleSwitch.vue'
import InfoTip from './InfoTip.vue'

const store = usePresetsStore()
const v = computed(() => store.currentConfig.video)
const out = computed(() => store.currentConfig.output)
const { expand } = useTemplateVars()

// Export video toggle. At least one of video/audio must stay enabled, so disabling
// video forces audio export on when it would otherwise leave nothing to produce.
function setVideoEnabled(on: boolean) {
  v.value.videoEnabled = on
  if (!on && !store.currentConfig.audioExport) {
    store.currentConfig.audioExport = { ...DEFAULT_AUDIO_EXPORT_CONFIG }
  }
}

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

const FRAMERATES: { value: Framerate; label: string }[] = [
  { value: 'source',  label: 'Original (unchanged)' },
  { value: '23.976', label: '23.976 fps (film NTSC)' },
  { value: '24',     label: '24 fps (cinema)' },
  { value: '25',     label: '25 fps (PAL)' },
  { value: '29.97',  label: '29.97 fps (NTSC)' },
  { value: '30',     label: '30 fps' },
  { value: '50',     label: '50 fps (PAL HFR)' },
  { value: '59.94',  label: '59.94 fps (NTSC HFR)' },
  { value: '60',     label: '60 fps' },
]

const ALL_HW_ACCELS: { value: HwAccel; label: string; os: string[] }[] = [
  { value: 'none',         label: 'Software (CPU)',           os: ['windows', 'linux', 'macos'] },
  { value: 'nvenc',        label: 'NVENC (Nvidia)',           os: ['windows', 'linux'] },
  { value: 'amf',          label: 'AMF (AMD)',                os: ['windows'] },
  { value: 'qsv',          label: 'Quick Sync (Intel)',       os: ['windows', 'linux'] },
  { value: 'videotoolbox', label: 'VideoToolbox (Apple)',     os: ['macos'] },
  { value: 'vaapi',        label: 'VAAPI (AMD/Intel Linux)',  os: ['linux'] },
]

const currentOs = ref('linux')
const HW_ACCELS = computed(() => ALL_HW_ACCELS.filter(h => h.os.includes(currentOs.value)))
const vaapiDevices = ref<string[]>([])

onMounted(async () => {
  currentOs.value = await platform()
  if (v.value.hwAccel !== 'none' && !HW_ACCELS.value.find(h => h.value === v.value.hwAccel)) {
    v.value.hwAccel = 'none'
  }
  if (currentOs.value === 'linux') {
    vaapiDevices.value = await invoke<string[]>('list_vaapi_devices')
    if (vaapiDevices.value.length && !vaapiDevices.value.includes(v.value.vaapiDevice)) {
      v.value.vaapiDevice = vaapiDevices.value[0]
    }
  }
})

const validContainers = computed(() => CODEC_CONTAINERS[v.value.codec] ?? [])
const showCrf = computed(() => v.value.codec !== 'copy')
const showEncodePreset = computed(() => !['libvp9', 'libsvtav1', 'copy'].includes(v.value.codec))
const showCustomRes = computed(() => v.value.resolution === 'custom')
const showResolution = computed(() => v.value.codec !== 'copy')
const showFramerate = computed(() => v.value.codec !== 'copy')
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
  } else if (v.value.encodePreset === null) {
    v.value.encodePreset = 'fast'
  }
}

// ── Filters: deinterlace (lives on video.deinterlace) ──
const di = computed(() => v.value.deinterlace)
const isCopy = computed(() => v.value.codec === 'copy')
const ALGOS: { value: DeinterlaceAlgo; label: string; desc: string }[] = [
  { value: 'bwdif', label: 'bwdif', desc: 'Best quality, motion-adaptive' },
  { value: 'yadif', label: 'yadif', desc: 'Fast, widely compatible' },
  { value: 'estdif', label: 'estdif', desc: 'Edge-slope tracing' },
]

// ── Output: video title + subfolder ──
const videoTitlePreview = computed(() =>
  out.value.nameOverride ? expand(out.value.nameOverride) : null,
)

const lastVideoSubdir = ref(out.value.videoSubdir || 'video')
const videoSubdirEnabled = ref(out.value.videoSubdir !== '')
watch(videoSubdirEnabled, (on) => {
  if (on) {
    out.value.videoSubdir = lastVideoSubdir.value
  } else {
    lastVideoSubdir.value = out.value.videoSubdir || lastVideoSubdir.value
    out.value.videoSubdir = ''
  }
})
</script>

<template>
  <div>
    <p class="section-title">Video</p>

    <div class="form-row">
      <span class="toggle-label">
        <ToggleSwitch :model-value="v.videoEnabled" @update:model-value="setVideoEnabled" />
        Export video file
      </span>
    </div>

    <div class="section-body" :class="{ disabled: !v.videoEnabled }">
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

    <div v-if="showFramerate" class="form-row">
      <label>Frame rate</label>
      <select v-model="v.framerate">
        <option v-for="f in FRAMERATES" :key="f.value" :value="f.value">{{ f.label }}</option>
      </select>
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
      <select v-model="v.vaapiDevice">
        <option v-for="d in vaapiDevices" :key="d" :value="d">{{ d }}</option>
      </select>
    </div>

    <!-- ── Filters ── -->
    <p class="sub-title">Filters</p>

    <div v-if="isCopy" class="notice">
      Deinterlace requires re-encoding. Select a codec above.
    </div>

    <template v-else>
      <div class="toggle-pair">
        <span class="toggle-label">
          <ToggleSwitch v-model="di.enabled" /> Deinterlace
        </span>
        <span class="toggle-label fade" :class="{ disabled: !di.enabled }">
          <ToggleSwitch v-model="di.autoDetect" /> Auto-detect
          <InfoTip>Filter skipped if the source is already progressive</InfoTip>
        </span>
      </div>

      <div class="section-body" :class="{ disabled: !di.enabled }">
        <div class="form-row">
          <label>Algorithm</label>
          <select v-model="di.algorithm">
            <option v-for="a in ALGOS" :key="a.value" :value="a.value">
              {{ a.label }} — {{ a.desc }}
            </option>
          </select>
        </div>
      </div>
    </template>

    <!-- ── Output ── -->
    <template v-if="!isCopy">
      <p class="sub-title">Output</p>

      <div class="form-row">
        <label>Video title</label>
        <div class="field-with-info">
          <input v-model="out.nameOverride" placeholder="Defaults to manual title" class="var-field" />
          <VarInfoButton />
        </div>
      </div>
      <p v-if="videoTitlePreview" class="filename-preview">→ <code>{{ videoTitlePreview }}.{{ v.container }}</code></p>

      <div class="form-row">
        <span class="toggle-label">
          <ToggleSwitch v-model="videoSubdirEnabled" /> Video subfolder
        </span>
        <div class="field-with-info">
          <input
            v-model="out.videoSubdir"
            placeholder="video"
            class="var-field subdir-input"
            :disabled="!videoSubdirEnabled"
            :class="{ disabled: !videoSubdirEnabled }"
          />
          <VarInfoButton />
        </div>
      </div>
    </template>
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

.toggle-pair {
  display: flex;
  align-items: center;
  gap: 28px;
  margin-bottom: 18px;
}

.section-body,
.toggle-label.fade {
  transition: opacity 0.1s ease;
}
.section-body.disabled,
.toggle-label.disabled {
  opacity: 0.4;
  pointer-events: none;
}

.notice {
  color: var(--muted);
  font-size: 12px;
  padding: 8px 12px;
  background: var(--elevated);
  border-radius: var(--radius-sm);
  border: 1px solid var(--border);
}

.hint { color: var(--muted); font-size: 12px; margin-top: -8px; margin-bottom: 6px; }

.filename-preview {
  font-size: 12px;
  color: var(--muted);
  margin-top: -12px;
  margin-bottom: 18px;
}
.filename-preview code { color: var(--text); font-family: monospace; font-size: 11px; word-break: break-all; }
</style>
