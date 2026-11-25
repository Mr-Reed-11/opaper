<template>
  <aside class="w-[300px] shrink-0 p-5 glass border-t-0 border-b-0 border-r-0 rounded-none flex flex-col gap-4 overflow-y-auto">
    <div class="text-xs tracking-widest uppercase text-[var(--text-muted)]">Opções do wallpaper</div>
    <div class="font-display font-bold text-xl">{{ wallpaper?.name || 'Selecione' }}</div>

    <template v-if="wallpaper">
      <!-- Dynamic manifest params -->
      <template v-if="manifestParams.length">
        <div class="text-xs text-[var(--text-muted)] mt-1">Parâmetros</div>
        <div class="flex flex-col gap-3">
          <template v-for="param in manifestParams" :key="param.key">
            <!-- Float slider -->
            <div v-if="param.type === 'float'" class="flex flex-col gap-1">
              <div class="flex justify-between text-[15px]">
                <span>{{ param.label }}</span>
                <span class="text-[var(--text-secondary)]">{{ paramDisplayValue(param) }}</span>
              </div>
              <input
                type="range"
                :min="param.min"
                :max="param.max"
                :step="paramStep(param)"
                :value="getParamFloat(param)"
                class="w-full h-1.5 rounded-full appearance-none bg-[rgba(255,255,255,0.14)] accent-[var(--accent)]"
                @input="onFloatChange(param, $event)"
              />
            </div>

            <!-- Color picker -->
            <div v-else-if="param.type === 'color'" class="flex flex-col gap-1">
              <div class="flex justify-between items-center text-[15px]">
                <span>{{ param.label }}</span>
                <div class="flex items-center gap-2">
                  <span class="text-[13px] text-[var(--text-secondary)]">{{ getParamHex(param) }}</span>
                  <label class="relative w-7 h-7 rounded-lg overflow-hidden border border-[var(--glass-border-active)] cursor-pointer">
                    <div class="absolute inset-0" :style="{ background: getParamHex(param) }" />
                    <input
                      type="color"
                      :value="getParamHex(param)"
                      class="absolute inset-0 opacity-0 cursor-pointer"
                      @input="onColorChange(param, $event)"
                    />
                  </label>
                </div>
              </div>
            </div>
          </template>
        </div>
      </template>

      <!-- Performance -->
      <div class="text-xs text-[var(--text-muted)] mt-1">Desempenho</div>
      <div class="flex flex-col gap-2 text-[15px]">
        <div class="flex justify-between">
          <span>Limite de FPS</span>
          <span class="text-[var(--text-secondary)]">{{ localFps }}</span>
        </div>
        <input
          type="range"
          min="15"
          max="144"
          step="1"
          v-model.number="localFps"
          class="w-full h-1.5 rounded-full appearance-none bg-[rgba(255,255,255,0.14)] accent-[var(--accent)]"
          @change="onFpsChange"
        />
        <ToggleRow label="Pausar com janela cheia" v-model="pauseFullscreen" />
      </div>

      <!-- Action buttons -->
      <div class="mt-auto flex flex-col gap-2.5">
        <button
          class="h-10 rounded-[10px] border border-[rgba(255,255,255,0.3)] bg-[var(--accent-hover)] font-semibold text-[15px] shadow-[inset_0_1px_0_rgba(255,255,255,0.3)]"
          @click="onApply"
        >
          Aplicar{{ output ? ` no ${output}` : '' }}
        </button>
        <button
          class="h-10 rounded-[10px] border border-[var(--glass-border-active)] bg-[var(--glass-bg)] text-[15px]"
          @click="onRestore"
        >
          Restaurar padrão
        </button>
      </div>
    </template>

    <div v-else class="flex-1 flex items-center justify-center text-sm text-[var(--text-muted)]">
      Clique em um wallpaper para ver as opções
    </div>
  </aside>
</template>

<script setup lang="ts">
import { ref, computed, watch, reactive } from 'vue'
import type { WallpaperInfo } from '../types'
import { useEngine } from '../composables/useEngine'
import { useWallpaperSettings } from '../composables/useWallpaperSettings'
import ToggleRow from './ToggleRow.vue'

interface ManifestParam {
  key: string
  type: string
  label: string
  slot: number
  default: any
  min?: number
  max?: number
}

const props = defineProps<{
  wallpaper: WallpaperInfo | null
  output: string
}>()

const emit = defineEmits<{
  apply: [overrides: Record<number, [number, number, number, number]>]
  restore: []
}>()

const { selectedManifest, activeParams, activeWallpaperPath, setParam, setFpsLimit, fpsLimit } = useEngine()
const { getOverrides, setOverrides, clearOverrides } = useWallpaperSettings()

const localFps = ref(fpsLimit.value)
const pauseFullscreen = ref(true)
const localParamOverrides = reactive<Record<number, [number, number, number, number]>>({})

watch(fpsLimit, (v) => { localFps.value = v })

watch(() => props.wallpaper, (wp) => {
  Object.keys(localParamOverrides).forEach(k => delete localParamOverrides[Number(k)])
  if (wp) {
    const saved = getOverrides(wp.path)
    for (const [slot, value] of Object.entries(saved)) {
      localParamOverrides[Number(slot)] = value
    }
  }
})

const manifestParams = computed<ManifestParam[]>(() => {
  if (!selectedManifest.value?.params) return []
  const params = selectedManifest.value.params as Record<string, any>
  return Object.entries(params)
    .map(([key, p]) => ({
      key,
      type: p.type,
      label: p.label || key,
      slot: p.slot,
      default: p.default,
      min: p.min,
      max: p.max,
    }))
    .sort((a, b) => a.slot - b.slot)
})

const isActive = computed(() =>
  props.wallpaper && activeWallpaperPath.value === props.wallpaper.path
)

function getParamFloat(param: ManifestParam): number {
  if (localParamOverrides[param.slot]) return localParamOverrides[param.slot][0]
  if (isActive.value && activeParams.value[param.slot]) return activeParams.value[param.slot][0]
  return typeof param.default === 'number' ? param.default : 0
}

function getParamHex(param: ManifestParam): string {
  let r: number, g: number, b: number
  if (localParamOverrides[param.slot]) {
    const v = localParamOverrides[param.slot]
    r = Math.round(v[0] * 255)
    g = Math.round(v[1] * 255)
    b = Math.round(v[2] * 255)
  } else if (isActive.value && activeParams.value[param.slot]) {
    const v = activeParams.value[param.slot]
    r = Math.round(v[0] * 255)
    g = Math.round(v[1] * 255)
    b = Math.round(v[2] * 255)
  } else if (typeof param.default === 'string') {
    return param.default
  } else {
    return '#000000'
  }
  return `#${r.toString(16).padStart(2, '0')}${g.toString(16).padStart(2, '0')}${b.toString(16).padStart(2, '0')}`
}

function paramStep(param: ManifestParam): number {
  if (param.min !== undefined && param.max !== undefined) {
    const range = param.max - param.min
    if (range <= 1) return 0.01
    if (range <= 10) return 0.1
    return 1
  }
  return 0.01
}

function paramDisplayValue(param: ManifestParam): string {
  const v = getParamFloat(param)
  if (param.max !== undefined && param.max > 50) return String(Math.round(v))
  return v.toFixed(2)
}

function onFloatChange(param: ManifestParam, event: Event) {
  const value = parseFloat((event.target as HTMLInputElement).value)
  localParamOverrides[param.slot] = [value, 0, 0, 1]
  if (isActive.value) {
    setParam(param.slot, [value, 0, 0, 1])
  }
}

function onColorChange(param: ManifestParam, event: Event) {
  const hex = (event.target as HTMLInputElement).value
  const r = parseInt(hex.slice(1, 3), 16) / 255
  const g = parseInt(hex.slice(3, 5), 16) / 255
  const b = parseInt(hex.slice(5, 7), 16) / 255
  localParamOverrides[param.slot] = [r, g, b, 1]
  if (isActive.value) {
    setParam(param.slot, [r, g, b, 1])
  }
}

function onApply() {
  if (props.wallpaper) {
    setOverrides(props.wallpaper.path, { ...localParamOverrides })
  }
  emit('apply', { ...localParamOverrides })
}

function onRestore() {
  if (props.wallpaper) {
    clearOverrides(props.wallpaper.path)
  }
  Object.keys(localParamOverrides).forEach(k => delete localParamOverrides[Number(k)])
  emit('restore')
}

function onFpsChange() {
  setFpsLimit(localFps.value)
}
</script>
