<template>
  <div class="flex gap-2 flex-wrap">
    <button
      v-for="tag in tags"
      :key="tag"
      class="px-3.5 py-1.5 rounded-full text-sm transition-colors border"
      :class="modelValue === tag
        ? 'bg-[var(--accent-hover)] font-semibold border-[rgba(255,255,255,0.25)]'
        : 'bg-[var(--glass-bg)] border-[var(--glass-border-active)]'"
      @click="$emit('update:modelValue', tag)"
    >
      {{ tag }}
    </button>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  tags: string[]
  modelValue: string
}>()

defineEmits<{ 'update:modelValue': [value: string] }>()
</script>
