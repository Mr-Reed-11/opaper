<template>
  <footer class="relative h-9 px-6 glass border-b-0 border-x-0 rounded-none flex items-center justify-between text-[13px] text-[var(--text-secondary)]">
    <span>{{ left }}</span>
    <span>{{ right }}</span>
  </footer>
</template>

<script setup lang="ts">
defineProps<{ left: string; right: string }>()
</script>
