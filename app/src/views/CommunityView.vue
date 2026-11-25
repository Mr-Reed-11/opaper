<template>
  <div class="flex-1 min-w-0 p-6 flex flex-col gap-[18px] overflow-y-auto">
    <div class="flex justify-between items-baseline">
      <h2 class="font-display font-bold text-[32px] tracking-tight">Baixar wallpapers</h2>
    </div>

    <div class="flex-1 flex flex-col items-center justify-center gap-4 text-[var(--text-muted)]">
      <Download :size="48" :stroke-width="1.5" />
      <p class="text-lg">Em breve</p>
      <p class="text-sm text-center max-w-xs">O catálogo da comunidade ainda está sendo construído. Fique ligado para novidades.</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Download } from '@lucide/vue'
</script>
