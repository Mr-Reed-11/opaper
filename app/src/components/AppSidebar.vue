<template>
  <nav class="w-[232px] shrink-0 px-4 py-6 glass border-t-0 border-b-0 border-l-0 rounded-none flex flex-col gap-1.5">
    <div class="text-xs tracking-widest uppercase text-[var(--text-muted)] px-3 pb-2">Menu</div>

    <router-link
      v-for="item in items"
      :key="item.to"
      :to="item.to"
      class="h-10 px-3 rounded-[10px] flex items-center gap-3 text-[15px] transition-colors"
      :class="$route.path === item.to
        ? 'glass-active font-semibold text-[var(--text-primary)]'
        : 'text-[var(--text-secondary)] hover:bg-[var(--glass-bg-hover)]'"
    >
      <component :is="item.icon" :size="18" />
      {{ item.label }}
    </router-link>
  </nav>
</template>

<script setup lang="ts">
import { LayoutGrid, Search, PenLine, Sun } from '@lucide/vue'

const items = [
  { to: '/', label: 'Todos', icon: LayoutGrid },
  { to: '/community', label: 'Baixar wallpapers', icon: Search },
  { to: '/my-wallpapers', label: 'Meus wallpapers', icon: PenLine },
  { to: '/settings', label: 'Configurações', icon: Sun },
]
</script>
