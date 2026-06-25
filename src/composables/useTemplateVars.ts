import { computed, ref } from 'vue'
import { usePresetsStore } from '@/stores/presets'

const CODEC_LABEL: Record<string, string> = {
  libx264: 'h264', libx265: 'h265', libvp9: 'vp9', libsvtav1: 'av1', copy: 'original',
}

// The currently-selected source file's title (filename stem), shared so template
// previews can resolve $ORIGINAL. Set from the main screen on file selection.
const sourceTitle = ref('')
export function setSourceTitle(title: string) { sourceTitle.value = title }

// Shared template-variable engine for output folder / title fields.
// Variables resolve against the live video config plus the current date/time.
export function useTemplateVars() {
  const store = usePresetsStore()
  const v = computed(() => store.currentConfig.video)

  const now = new Date()
  const pad = (n: number) => String(n).padStart(2, '0')
  const dateStr     = `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}`
  const timeStr     = `${pad(now.getHours())}-${pad(now.getMinutes())}-${pad(now.getSeconds())}`
  const datetimeStr = `${dateStr}_${timeStr}`

  const resolutionLabel = () =>
    v.value.resolution === 'custom'
      ? `${v.value.customWidth ?? 0}x${v.value.customHeight ?? 0}`
      : v.value.resolution

  const expand = (s: string) =>
    s
      .replace(/\$ORIGINAL/g,   sourceTitle.value)
      .replace(/\$DATETIME/g,   datetimeStr)
      .replace(/\$DATE/g,       dateStr)
      .replace(/\$TIME/g,       timeStr)
      .replace(/\$CODEC/g,      CODEC_LABEL[v.value.codec] ?? v.value.codec)
      .replace(/\$RESOLUTION/g, resolutionLabel())
      .replace(/\$FRAMERATE/g,  v.value.framerate)
      .replace(/\$CRF/g,        String(v.value.crf))
      .replace(/\$PRESET/g,     v.value.encodePreset ?? '')
      .replace(/\$DI/g,         v.value.deinterlace.enabled ? 'DI' : '')

  const VAR_REGEX = /\$(ORIGINAL|DATE|TIME|DATETIME|CODEC|RESOLUTION|FRAMERATE|CRF|PRESET|DI)/

  const hasVars = (s: string) => VAR_REGEX.test(s)

  const VARS = computed(() => [
    { name: '$ORIGINAL',   desc: 'Original video title',                example: sourceTitle.value || '(input filename)' },
    { name: '$DATE',       desc: 'Current date',                        example: dateStr },
    { name: '$TIME',       desc: 'Current time (HH-MM-SS)',             example: timeStr },
    { name: '$DATETIME',   desc: 'Date and time combined',             example: datetimeStr },
    { name: '$CODEC',      desc: 'Video codec',                         example: CODEC_LABEL[v.value.codec] ?? v.value.codec },
    { name: '$RESOLUTION', desc: 'Output resolution',                   example: resolutionLabel() },
    { name: '$FRAMERATE',  desc: 'Output frame rate',                   example: v.value.framerate },
    { name: '$CRF',        desc: 'Quality (CRF) value',                 example: String(v.value.crf) },
    { name: '$PRESET',     desc: 'Encode speed preset',                 example: v.value.encodePreset ?? '(none)' },
    { name: '$DI',         desc: '"DI" when deinterlacing, else empty', example: v.value.deinterlace.enabled ? 'DI' : '(empty)' },
  ])

  return { expand, hasVars, VAR_REGEX, VARS, dateStr, timeStr, datetimeStr }
}
