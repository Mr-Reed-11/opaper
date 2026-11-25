<template>
  <div class="flex-1 min-w-0 p-6 flex flex-col gap-5 overflow-y-auto">
    <MonitorSelector
      :outputs="outputs"
      v-model="selectedOutput"
    />

    <div class="flex justify-between items-baseline">
      <h2 class="font-display font-bold text-[32px] tracking-tight">Wallpapers</h2>
      <span class="text-[15px] text-[var(--text-secondary)]">
        {{ wallpapers.length }} itens{{ selectedOutput ? ` · Monitor ${selectedOutput}` : '' }}{{ activeWallpaperPath ? ' em uso' : '' }}
      </span>
    </div>

    <WallpaperGrid>
      <WallpaperCard
        v-for="wp in wallpapers"
        :key="wp.path"
        :name="wp.name"
        :author="wp.author"
        :tags="wallpaperTags(wp)"
        :preview-color="wallpaperColor(wp)"
        :active="activeWallpaperPath === wp.path"
        @select="onSelect(wp)"
      />
    </WallpaperGrid>

    <p v-if="!wallpapers.length" class="py-20 text-center text-sm text-[var(--text-muted)]">
      Nenhum wallpaper encontrado.
    </p>
  </div>

  <WallpaperOptions
    :wallpaper="selected"
    :output="selectedOutput"
    @apply="onApply($event)"
    @restore="onRestore"
  />
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useEngine } from '../composables/useEngine'
import type { WallpaperInfo } from '../types'
import MonitorSelector from '../components/MonitorSelector.vue'
import WallpaperGrid from '../components/WallpaperGrid.vue'
import WallpaperCard from '../components/WallpaperCard.vue'
import WallpaperOptions from '../components/WallpaperOptions.vue'

const {
  wallpapers, outputs, selectedOutput, activeWallpaperPath,
  applyWallpaper, setParam, restoreDefault, loadManifestFor, init,
} = useEngine()

const selected = ref<WallpaperInfo | null>(null)

function wallpaperColor(wp: WallpaperInfo): string {
  return wp.colors[0] || '#8B5CF6'
}

function wallpaperTags(wp: WallpaperInfo): string[] {
  return ['Shader']
}

async function onSelect(wp: WallpaperInfo) {
  selected.value = wp
  await loadManifestFor(wp)
}

async function onApply(overrides: Record<number, [number, number, number, number]>) {
  if (selected.value) {
    await applyWallpaper(selected.value)
    for (const [slot, value] of Object.entries(overrides)) {
      await setParam(Number(slot), value)
    }
  }
}

async function onRestore() {
  selected.value = null
  await restoreDefault()
}

onMounted(init)
</script>
