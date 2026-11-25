<template>
  <button
    class="flex flex-col gap-2 p-2.5 rounded-[18px] glass glass-shadow text-left transition-all"
    :class="active ? 'border-2 border-[rgba(255,255,255,0.9)]' : ''"
    @click="$emit('select')"
  >
    <div
      class="h-[130px] w-full rounded-xl relative"
      :style="{ background: previewColor }"
    >
      <span
        v-if="tags.length"
        class="absolute left-2.5 bottom-2.5 px-2.5 py-1 rounded-full text-xs font-semibold border border-[rgba(255,255,255,0.2)]"
        :style="{ background: 'rgba(14,22,64,0.55)', color: 'var(--text-primary)' }"
      >
        {{ tags.join(' · ') }}
      </span>
    </div>

    <div class="text-base font-semibold">{{ name }}</div>
    <div class="text-sm text-[var(--text-secondary)]">
      {{ author }}{{ resolution ? ` · ${resolution}` : '' }}{{ active ? ' · em uso' : '' }}
    </div>
  </button>
</template>

<script setup lang="ts">
defineProps<{
  name: string
  author: string
  resolution?: string
  tags: string[]
  previewColor: string
  active: boolean
}>()

defineEmits<{ select: [] }>()
</script>
