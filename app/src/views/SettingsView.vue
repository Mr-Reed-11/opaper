<template>
  <div class="flex-1 min-w-0 p-6 flex flex-col gap-5 overflow-y-auto">
    <h2 class="font-display font-bold text-[32px] tracking-tight">Configurações</h2>

    <div class="grid grid-cols-2 gap-4">
      <SettingsCard title="Geral" subtitle="Comportamento do aplicativo" :icon="Settings2">
        <div class="flex flex-col gap-3 text-[15px]">
          <ToggleRow label="Iniciar com o sistema" v-model="autostart" />
          <ToggleRow label="Minimizar para bandeja ao fechar" v-model="trayOnClose" />
          <div class="flex justify-between items-center">
            <span>Tema</span>
            <button
              class="px-3 py-1 rounded-lg bg-[var(--glass-bg)] border border-[var(--glass-border-active)] text-sm"
              @click="toggleTheme"
            >
              {{ theme === 'blue' ? 'Azul e roxo' : 'Preto e branco' }}
            </button>
          </div>
          <div class="flex justify-between items-center">
            <span>Idioma</span>
            <span class="text-[var(--text-secondary)]">Português (BR)</span>
          </div>
        </div>
      </SettingsCard>

      <SettingsCard title="Desempenho" subtitle="GPU e consumo de energia" :icon="Gauge">
        <div class="flex flex-col gap-3 text-[15px]">
          <div class="flex justify-between">
            <span>Limite global de FPS</span>
            <span class="text-[var(--text-secondary)]">{{ localFps }}</span>
          </div>
          <input
            type="range" min="15" max="144" step="1" v-model.number="localFps"
            class="w-full h-1.5 rounded-full appearance-none bg-[rgba(255,255,255,0.14)] accent-[var(--accent)]"
            @change="onFpsChange"
          />
          <ToggleRow label="Pausar quando em bateria" v-model="pauseBattery" />
          <ToggleRow label="Pausar com janela em tela cheia" v-model="pauseFullscreen" />
        </div>
      </SettingsCard>

      <SettingsCard title="Monitores" subtitle="Saídas de vídeo detectadas" :icon="MonitorIcon">
        <div class="flex flex-col gap-2 text-[15px]">
          <div
            v-for="output in outputs"
            :key="output"
            class="flex items-center gap-3 px-3 py-2 rounded-xl bg-[var(--glass-bg)] border border-[var(--glass-border)]"
          >
            <MonitorIcon :size="18" class="text-[var(--text-secondary)]" />
            <span>{{ output }}</span>
          </div>
          <p v-if="!outputs.length" class="text-sm text-[var(--text-muted)]">Nenhum monitor detectado</p>
        </div>
      </SettingsCard>

      <SettingsCard title="Interação e áudio" subtitle="Reação a som e entrada" :icon="Volume2">
        <div class="flex flex-col gap-3 text-[15px]">
          <ToggleRow label="Reagir ao áudio do sistema" v-model="audioReact" />
          <ToggleRow label="Reagir ao movimento do mouse" v-model="mouseReact" />
        </div>
      </SettingsCard>

      <SettingsCard title="Armazenamento" subtitle="Wallpapers baixados e cache" :icon="HardDrive" class="col-span-2">
        <div class="flex flex-col gap-3 text-[15px]">
          <div class="flex justify-between">
            <span>Wallpapers locais</span>
            <span class="text-[var(--text-secondary)]">{{ wallpapers.length }} itens</span>
          </div>
          <div class="flex justify-between">
            <span>Cache de shaders</span>
            <span class="text-[var(--text-secondary)]">—</span>
          </div>
          <button class="self-start px-4 py-2 rounded-xl bg-[var(--glass-bg)] border border-[var(--glass-border-active)] text-sm">
            Limpar cache
          </button>
        </div>
      </SettingsCard>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { Settings2, Gauge, Monitor as MonitorIcon, Volume2, HardDrive } from '@lucide/vue'
import { useEngine } from '../composables/useEngine'
import { useTheme } from '../composables/useTheme'
import SettingsCard from '../components/SettingsCard.vue'
import ToggleRow from '../components/ToggleRow.vue'

const { outputs, wallpapers, fpsLimit, setFpsLimit } = useEngine()
const { theme, toggle: toggleTheme } = useTheme()

const localFps = ref(fpsLimit.value)
const autostart = ref(true)
const trayOnClose = ref(true)
const pauseBattery = ref(false)
const pauseFullscreen = ref(true)
const audioReact = ref(false)
const mouseReact = ref(false)

watch(fpsLimit, (v) => { localFps.value = v })

function onFpsChange() {
  setFpsLimit(localFps.value)
}
</script>
