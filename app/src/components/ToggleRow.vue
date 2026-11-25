<template>
  <div class="flex justify-between items-center">
    <span>{{ label }}</span>
    <button
      role="switch"
      :aria-checked="modelValue"
      class="w-10 h-[22px] rounded-full relative transition-colors"
      :class="modelValue
        ? 'bg-[var(--switch-on)]'
        : 'bg-[var(--switch-off-bg)] border border-[var(--switch-off-border)]'"
      @click="$emit('update:modelValue', !modelValue)"
    >
      <span
        class="absolute top-[3px] w-4 h-4 rounded-full transition-all"
        :class="modelValue
          ? 'right-[3px] bg-[var(--text-primary)]'
          : 'left-[3px] bg-[var(--text-secondary)]'"
      />
    </button>
  </div>
</template>

<script setup lang="ts">
defineProps<{ label: string; modelValue: boolean }>()
defineEmits<{ 'update:modelValue': [value: boolean] }>()
</script>
