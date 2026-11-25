<template>
  <div
    v-if="outputs.length"
    role="radiogroup"
    aria-label="Monitor de destino"
    class="flex items-center gap-3 p-2 self-start rounded-[14px] glass glass-shadow"
  >
    <span class="px-2 pl-1.5 text-[13px] text-[var(--text-muted)]">Aplicar em</span>

    <button
      role="radio"
      :aria-checked="modelValue === ''"
      class="flex items-center gap-2.5 px-3.5 py-2 rounded-[10px] text-sm transition-colors"
      :class="modelValue === ''
        ? 'bg-[var(--accent-hover)] border border-[rgba(255,255,255,0.3)] font-semibold shadow-[inset_0_1px_0_rgba(255,255,255,0.3)]'
        : 'bg-[var(--glass-bg)] border border-[var(--glass-border)] text-[var(--text-secondary)]'"
      @click="$emit('update:modelValue', '')"
    >
      <MonitorIcon :size="18" />
      Todos
      <Check v-if="modelValue === ''" :size="16" :stroke-width="3" />
    </button>

    <button
      v-for="output in outputs"
      :key="output"
      role="radio"
      :aria-checked="modelValue === output"
      class="flex items-center gap-2.5 px-3.5 py-2 rounded-[10px] text-sm transition-colors"
      :class="modelValue === output
        ? 'bg-[var(--accent-hover)] border border-[rgba(255,255,255,0.3)] font-semibold shadow-[inset_0_1px_0_rgba(255,255,255,0.3)]'
        : 'bg-[var(--glass-bg)] border border-[var(--glass-border)] text-[var(--text-secondary)]'"
      @click="$emit('update:modelValue', output)"
    >
      <MonitorIcon :size="18" />
      {{ output }}
      <Check v-if="modelValue === output" :size="16" :stroke-width="3" />
    </button>
  </div>
</template>

<script setup lang="ts">
import { Monitor as MonitorIcon, Check } from '@lucide/vue'

defineProps<{
  outputs: string[]
  modelValue: string
}>()

defineEmits<{
  'update:modelValue': [value: string]
}>()
</script>
