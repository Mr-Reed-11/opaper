<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { ref } from 'vue';
import { FolderOpen, Monitor, Pause, Play, RefreshCw } from '@lucide/vue';

const outputs = ref<string[]>([]);
const path = ref('');
const message = ref('');
const paused = ref(false);

async function refresh() {
  try {
    outputs.value = await invoke<string[]>('list_outputs');
    message.value = '';
  } catch (error) {
    message.value = `Engine indisponível: ${error}`;
  }
}

async function apply() {
  if (!path.value.trim()) return;
  try {
    await invoke('set_wallpaper', { path: path.value.trim() });
    message.value = 'Wallpaper aplicado.';
  } catch (error) {
    message.value = `Falha ao aplicar: ${error}`;
  }
}

async function togglePause() {
  try {
    await invoke(paused.value ? 'resume' : 'pause');
    paused.value = !paused.value;
  } catch (error) {
    message.value = `Falha ao controlar a engine: ${error}`;
  }
}

refresh();
</script>

<template>
  <main class="mx-auto max-w-4xl p-8">
    <header class="mb-8 flex items-center justify-between">
      <div class="flex items-center gap-3">
        <FolderOpen class="size-7 text-indigo-400" />
        <h1 class="text-2xl font-semibold tracking-tight">opaper</h1>
      </div>
      <button
        class="flex items-center gap-2 rounded-lg border border-white/10 px-3 py-2 text-sm hover:bg-white/5"
        @click="refresh"
      >
        <RefreshCw class="size-4" /> Atualizar
      </button>
    </header>

    <section class="rounded-xl border border-white/10 bg-white/5 p-6">
      <h2 class="mb-4 text-sm font-medium text-white/60">Biblioteca</h2>
      <label class="block text-sm text-white/70" for="wallpaper-path">Pasta do wallpaper</label>
      <div class="mt-2 flex gap-2">
        <input
          id="wallpaper-path"
          v-model="path"
          class="min-w-0 flex-1 rounded-lg border border-white/10 bg-black/30 px-3 py-2 text-sm outline-none focus:border-indigo-400"
          placeholder="/caminho/para/plasma-waves"
        />
        <button
          class="rounded-lg bg-indigo-500 px-4 py-2 text-sm font-medium text-white hover:bg-indigo-400"
          @click="apply"
        >
          Aplicar
        </button>
        <button
          class="rounded-lg border border-white/10 px-4 py-2 text-sm hover:bg-white/5"
          @click="togglePause"
        >
          <Pause v-if="!paused" class="mr-1 inline size-4" />
          <Play v-else class="mr-1 inline size-4" />
          {{ paused ? 'Retomar' : 'Pausar' }}
        </button>
      </div>
      <p v-if="message" class="mt-4 text-sm text-amber-300">{{ message }}</p>
    </section>

    <section class="mt-6 rounded-xl border border-white/10 bg-white/5 p-6">
      <h2 class="mb-4 flex items-center gap-2 text-sm font-medium text-white/60">
        <Monitor class="size-4" /> Outputs
      </h2>
      <ul class="space-y-2 text-sm">
        <li v-for="output in outputs" :key="output" class="rounded-lg bg-black/30 px-3 py-2">
          {{ output }}
        </li>
        <li v-if="!outputs.length" class="text-white/40">Nenhum output detectado.</li>
      </ul>
    </section>
  </main>
</template>
