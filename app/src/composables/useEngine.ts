import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { WallpaperInfo } from '../types'

const wallpapers = ref<WallpaperInfo[]>([])
const outputs = ref<string[]>([])
const selectedOutput = ref('')
const activeWallpaperPath = ref('')
const selectedManifest = ref<Record<string, any> | null>(null)
const activeParams = ref<number[][]>([])
const paused = ref(false)
const connected = ref(false)
const fpsLimit = ref(60)
const pauseOnFullscreen = ref(true)


async function loadWallpapers() {
  try {
    wallpapers.value = await invoke<WallpaperInfo[]>('list_wallpapers')
  } catch { /* engine off */ }
}

async function loadOutputs() {
  try {
    const all = await invoke<string[]>('list_outputs')
    outputs.value = all.filter(o => o.length > 0)
    connected.value = true
  } catch {
    connected.value = false
  }
}

async function loadActiveState() {
  try {
    activeWallpaperPath.value = await invoke<string>('get_active_wallpaper')
    const paramsJson = await invoke<string>('get_params')
    if (paramsJson) {
      activeParams.value = JSON.parse(paramsJson)
    }
  } catch { /* engine off */ }
}

async function loadManifestFor(wp: WallpaperInfo) {
  try {
    const json = await invoke<string>('read_manifest', { path: wp.path })
    selectedManifest.value = JSON.parse(json)
  } catch {
    selectedManifest.value = null
  }
}

async function applyWallpaper(wp: WallpaperInfo) {
  await invoke('set_wallpaper', {
    path: wp.path,
    output: selectedOutput.value || null,
  })
  activeWallpaperPath.value = wp.path
  await loadActiveState()
}

async function setParam(slot: number, value: [number, number, number, number]) {
  await invoke('set_param', {
    slot,
    r: value[0],
    g: value[1],
    b: value[2],
    a: value[3],
    output: selectedOutput.value || null,
  })
  if (activeParams.value[slot]) {
    activeParams.value[slot] = [...value]
  }
}

async function setFpsLimit(fps: number) {
  await invoke('set_fps_limit', { fps })
  fpsLimit.value = fps
}

async function setPauseOnFullscreen(enabled: boolean) {
  await invoke('set_pause_on_fullscreen', { enabled })
  pauseOnFullscreen.value = enabled
}

async function restoreDefault() {
  activeWallpaperPath.value = ''
  selectedManifest.value = null
  activeParams.value = []
}

async function togglePause() {
  await invoke(paused.value ? 'resume' : 'pause')
  paused.value = !paused.value
}

async function quitApp() {
  await invoke('quit')
}

async function init() {
  await Promise.all([loadWallpapers(), loadOutputs()])
  await loadActiveState()
}

export function useEngine() {
  return {
    wallpapers,
    outputs,
    selectedOutput,
    activeWallpaperPath,
    selectedManifest,
    activeParams,
    paused,
    connected,
    fpsLimit,
    pauseOnFullscreen,
    loadWallpapers,
    loadOutputs,
    loadActiveState,
    loadManifestFor,
    applyWallpaper,
    setParam,
    setFpsLimit,
    setPauseOnFullscreen,
    restoreDefault,
    togglePause,
    quitApp,
    init,
  }
}
