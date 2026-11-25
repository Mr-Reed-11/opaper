<script setup lang="ts">
import { useEngine } from './composables/useEngine'
import AppHeader from './components/AppHeader.vue'
import AppSidebar from './components/AppSidebar.vue'
import StatusBar from './components/StatusBar.vue'

const { connected } = useEngine()
</script>

<template>
  <div class="relative h-screen flex flex-col overflow-hidden">
    <!-- Background glows -->
    <div class="pointer-events-none absolute -right-[160px] -top-[160px] w-[520px] h-[520px] rounded-full bg-[var(--bg-glow-1)] opacity-45 blur-[110px]" />
    <div class="pointer-events-none absolute left-[260px] bottom-0 w-[560px] h-[460px] rounded-full bg-[var(--bg-glow-2)] opacity-45 blur-[120px]" />

    <AppHeader />

    <div class="relative flex flex-1 min-h-0">
      <AppSidebar />
      <router-view />
    </div>

    <StatusBar
      :left="`Plasma 6 · Wayland · engine ${connected ? 'conectada' : 'desconectada'}`"
      right="Clique em um wallpaper para ver as opções"
    />
  </div>
</template>
