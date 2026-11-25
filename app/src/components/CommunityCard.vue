<template>
  <div class="flex flex-col gap-2 p-2.5 rounded-[18px] glass glass-shadow">
    <div
      class="h-[130px] rounded-xl relative"
      :style="{ background: color }"
    >
      <span
        class="absolute left-2.5 bottom-2.5 px-2.5 py-1 rounded-full text-xs font-semibold border border-[rgba(255,255,255,0.2)]"
        style="background: rgba(14,22,64,0.55)"
      >
        {{ tags.join(' · ') }}
      </span>
    </div>

    <div class="text-base font-semibold">{{ name }}</div>
    <div class="text-sm text-[var(--text-secondary)]">por {{ author }} · {{ resolution }}</div>

    <div class="flex justify-between items-center text-[13px] text-[var(--text-muted)]">
      <span>{{ rating }} ★ · {{ formattedDownloads }} downloads</span>
      <button
        v-if="available"
        class="px-3 py-1.5 rounded-lg bg-[var(--accent-hover)] text-[var(--text-primary)] font-semibold text-sm"
      >
        Baixar
      </button>
      <span
        v-else
        class="px-3 py-1.5 rounded-lg bg-[var(--glass-bg)] border border-[var(--glass-border-active)] text-[var(--text-secondary)] font-semibold text-sm"
      >
        Em breve
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  name: string
  author: string
  resolution: string
  tags: string[]
  rating: number
  downloads: number
  color: string
  available: boolean
}>()

const formattedDownloads = computed(() => {
  if (props.downloads >= 1000) {
    return `${(props.downloads / 1000).toFixed(1).replace('.0', '')} mil`
  }
  return String(props.downloads)
})
</script>
