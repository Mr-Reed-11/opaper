<template>
  <div class="flex-1 min-w-0 p-6 flex flex-col gap-5 overflow-y-auto">
    <div class="flex justify-between items-baseline">
      <h2 class="font-display font-bold text-[32px] tracking-tight">Meus wallpapers</h2>
      <span class="text-[15px] text-[var(--text-secondary)]">{{ userWallpapers.length }} itens</span>
    </div>

    <WallpaperGrid>
      <WallpaperCard
        v-for="wp in userWallpapers"
        :key="wp.path"
        :name="wp.name"
        :author="wp.author || 'por você'"
        :tags="['Shader']"
        :preview-color="wp.colors[0] || '#8B5CF6'"
        :active="false"
        @select="() => {}"
      />
    </WallpaperGrid>

    <p v-if="!userWallpapers.length" class="py-20 text-center text-sm text-[var(--text-muted)]">
      Você ainda não criou nenhum wallpaper.
    </p>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useEngine } from '../composables/useEngine'
import WallpaperGrid from '../components/WallpaperGrid.vue'
import WallpaperCard from '../components/WallpaperCard.vue'

const { wallpapers } = useEngine()

const userWallpapers = computed(() =>
  wallpapers.value.filter(wp => wp.author === '' || wp.author.toLowerCase().includes('você'))
)
</script>
