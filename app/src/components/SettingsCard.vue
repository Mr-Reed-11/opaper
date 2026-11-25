<template>
  <div class="glass glass-shadow rounded-2xl p-5 flex flex-col gap-4" :class="$attrs.class">
    <div class="flex items-center gap-3">
      <div class="w-9 h-9 rounded-xl bg-[var(--accent-hover)] flex items-center justify-center">
        <component :is="icon" :size="20" />
      </div>
      <div>
        <div class="font-semibold text-base">{{ title }}</div>
        <div class="text-sm text-[var(--text-secondary)]">{{ subtitle }}</div>
      </div>
    </div>
    <slot />
  </div>
</template>

<script setup lang="ts">
import type { Component } from 'vue'

defineOptions({ inheritAttrs: false })

defineProps<{
  title: string
  subtitle: string
  icon: Component
}>()
</script>
