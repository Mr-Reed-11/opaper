<template>
  <button
    class="px-2.5 py-1 rounded-full text-xs font-semibold transition-colors"
    :class="modelValue
      ? 'bg-[var(--accent-hover)]'
      : 'bg-[var(--glass-bg)] border border-[var(--glass-border-active)]'"
    @click="$emit('update:modelValue', !modelValue)"
  >
    {{ label }}
  </button>
</template>

<script setup lang="ts">
defineProps<{ label: string; modelValue: boolean }>()
defineEmits<{ 'update:modelValue': [value: boolean] }>()
</script>
