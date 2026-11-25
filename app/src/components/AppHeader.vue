<template>
  <header class="relative h-14 px-6 flex items-center justify-between glass border-t-0 border-x-0 rounded-none">
    <div class="flex items-center gap-2.5">
      <AppLogo :size="28" />
      <span class="font-display font-bold text-[22px] tracking-tight">opaper</span>
    </div>

    <div class="flex gap-3 items-center">
      <div class="w-[300px] h-9 rounded-[10px] bg-[var(--glass-bg)] border border-[var(--glass-border-active)] px-3 flex items-center gap-2 text-[var(--text-secondary)] text-[15px]">
        <Search :size="16" class="shrink-0" />
        <span>Buscar na biblioteca</span>
      </div>

      <button class="w-9 h-9 rounded-[10px] bg-[var(--glass-bg-hover)] border border-[var(--glass-border-active)] flex items-center justify-center" title="Baixados">
        <Download :size="18" />
      </button>

      <button class="h-9 px-4 rounded-[10px] border border-[rgba(255,255,255,0.3)] bg-[var(--accent-hover)] font-semibold text-[15px] shadow-[inset_0_1px_0_rgba(255,255,255,0.3)]">
        Criar wallpaper
      </button>
    </div>
  </header>
</template>

<script setup lang="ts">
import { Search, Download } from '@lucide/vue'
import AppLogo from './AppLogo.vue'
</script>
