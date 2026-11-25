import { reactive } from 'vue'

type ParamOverrides = Record<number, [number, number, number, number]>

const STORAGE_KEY = 'opaper-wallpaper-settings'

function loadAll(): Record<string, ParamOverrides> {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    return raw ? JSON.parse(raw) : {}
  } catch {
    return {}
  }
}

function saveAll(data: Record<string, ParamOverrides>) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(data))
}

const cache = reactive<Record<string, ParamOverrides>>(loadAll())

export function useWallpaperSettings() {
  function getOverrides(wallpaperPath: string): ParamOverrides {
    return cache[wallpaperPath] || {}
  }

  function setOverrides(wallpaperPath: string, overrides: ParamOverrides) {
    if (Object.keys(overrides).length === 0) {
      delete cache[wallpaperPath]
    } else {
      cache[wallpaperPath] = { ...overrides }
    }
    saveAll({ ...cache })
  }

  function clearOverrides(wallpaperPath: string) {
    delete cache[wallpaperPath]
    saveAll({ ...cache })
  }

  return { getOverrides, setOverrides, clearOverrides }
}
