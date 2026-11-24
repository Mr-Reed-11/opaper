<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { ref, onMounted, computed } from 'vue';
import { Monitor, Pause, Play, Power } from '@lucide/vue';

interface WallpaperInfo {
  name: string;
  description: string;
  author: string;
  path: string;
  colors: string[];
}

const wallpapers = ref<WallpaperInfo[]>([]);
const outputs = ref<string[]>([]);
const selectedOutput = ref('');
const activeWallpaper = ref('');
const message = ref('');
const paused = ref(false);
const hoveredIndex = ref<number | null>(null);

function wallpaperGradient(colors: string[]): string {
  if (colors.length === 0) return 'linear-gradient(135deg, #1a1a2e, #16213e)';
  if (colors.length === 1) return `linear-gradient(135deg, ${colors[0]}, ${colors[0]}88)`;
  const stops = colors.map((c, i) => `${c} ${(i / (colors.length - 1)) * 100}%`);
  return `linear-gradient(135deg, ${stops.join(', ')})`;
}

function wallpaperGradientAnimated(colors: string[]): string {
  if (colors.length === 0) return 'linear-gradient(135deg, #1a1a2e, #16213e)';
  const doubled = [...colors, ...colors.slice().reverse()];
  const stops = doubled.map((c, i) => `${c} ${(i / (doubled.length - 1)) * 100}%`);
  return `linear-gradient(135deg, ${stops.join(', ')})`;
}

async function loadData() {
  try {
    wallpapers.value = await invoke<WallpaperInfo[]>('list_wallpapers');
  } catch { /* engine might be off */ }
  try {
    const allOutputs = await invoke<string[]>('list_outputs');
    // Filter out empty strings and add "All" option
    outputs.value = allOutputs.filter(o => o.length > 0).map(o => o);
    selectedOutput.value = outputs.value[0] || '';
    message.value = '';
  } catch {
    message.value = 'Engine não conectada';
  }
}

async function applyWallpaper(wp: WallpaperInfo) {
  try {
    await invoke('set_wallpaper', {
      path: wp.path,
      output: selectedOutput.value || null,
    });
    activeWallpaper.value = wp.path;
    message.value = '';
  } catch (error) {
    message.value = `Falha: ${error}`;
  }
}

async function togglePause() {
  try {
    await invoke(paused.value ? 'resume' : 'pause');
    paused.value = !paused.value;
  } catch (error) {
    message.value = `${error}`;
  }
}

onMounted(loadData);
</script>

<template>
  <main class="min-h-screen">
    <header class="sticky top-0 z-10 border-b border-white/10 bg-[#0b0e16]/90 backdrop-blur-lg">
      <div class="mx-auto flex max-w-6xl items-center justify-between px-6 py-4">
        <div class="flex items-center gap-3">
          <div class="flex size-8 items-center justify-center rounded-lg bg-indigo-500/20">
            <span class="text-lg font-bold text-indigo-400">O</span>
          </div>
          <h1 class="text-lg font-semibold tracking-tight">opaper</h1>
        </div>

        <div class="flex items-center gap-3">
          <div v-if="outputs.length" class="flex items-center gap-2">
            <Monitor class="size-4 text-white/40" />
            <select
              v-model="selectedOutput"
              class="rounded-lg border border-white/10 bg-white/5 px-3 py-1.5 text-sm outline-none focus-border-indigo-400"
            >
              <option value="">Todos os outputs</option>
              <option v-for="o in outputs" :key="o" :value="o">{{ o }}</option>
            </select>
          </div>

          <button
            class="flex items-center gap-1.5 rounded-lg border border-white/10 px-3 py-1.5 text-sm transition-colors hover:bg-white/5"
            @click="togglePause"
          >
            <Pause v-if="!paused" class="size-3.5" />
            <Play v-else class="size-3.5" />
            {{ paused ? 'Play' : 'Pause' }}
          </button>

        </div>
      </div>
    </header>

    <div class="mx-auto max-w-6xl px-6 py-6">
      <p v-if="message" class="mb-4 rounded-lg bg-amber-500/10 px-4 py-2 text-sm text-amber-300">
        {{ message }}
      </p>

      <div class="grid grid-cols-2 gap-4 sm:grid-cols-3 lg:grid-cols-4">
        <button
          v-for="(wp, i) in wallpapers"
          :key="wp.path"
          class="group relative overflow-hidden rounded-xl border transition-all duration-300"
          :class="activeWallpaper === wp.path
            ? 'border-indigo-400 ring-2 ring-indigo-400/30'
            : 'border-white/10 hover:border-white/25'"
          @click="applyWallpaper(wp)"
          @mouseenter="hoveredIndex = i"
          @mouseleave="hoveredIndex = null"
        >
          <div
            class="aspect-video w-full transition-transform duration-500 group-hover:scale-110"
            :style="{
              background: hoveredIndex === i
                ? wallpaperGradientAnimated(wp.colors)
                : wallpaperGradient(wp.colors),
              backgroundSize: hoveredIndex === i ? '200% 200%' : '100% 100%',
              animation: hoveredIndex === i ? 'gradientShift 3s ease infinite' : 'none',
            }"
          />

          <div class="absolute inset-0 bg-gradient-to-t from-black/80 via-black/20 to-transparent" />

          <div class="absolute bottom-0 left-0 right-0 p-3">
            <p class="text-sm font-medium text-white">{{ wp.name }}</p>
            <p class="mt-0.5 line-clamp-1 text-xs text-white/50">{{ wp.description }}</p>
          </div>

          <div
            v-if="activeWallpaper === wp.path"
            class="absolute right-2 top-2 rounded-full bg-indigo-500 px-2 py-0.5 text-[10px] font-medium text-white"
          >
            Ativo
          </div>
        </button>
      </div>

      <p v-if="!wallpapers.length" class="py-20 text-center text-sm text-white/30">
        Nenhum wallpaper encontrado.
      </p>
    </div>
  </main>
</template>

<style>
@keyframes gradientShift {
  0%, 100% { background-position: 0% 50%; }
  50% { background-position: 100% 50%; }
}
</style>
