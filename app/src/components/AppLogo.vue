<template>
  <svg :width="size" :height="size" viewBox="0 0 1024 1024" aria-hidden="true" class="shrink-0">
    <rect x="6" y="6" width="1012" height="1012" rx="226"
      :fill="'var(--logo-bg)'" :stroke="'var(--logo-stroke)'" stroke-width="24" />
    <g transform="translate(32 32)">
      <rect x="200" y="200" width="380" height="380" rx="84"
        :fill="'var(--logo-rect-1)'" fill-opacity="0.6" />
      <rect x="290" y="290" width="380" height="380" rx="84"
        :fill="'var(--logo-rect-2)'" fill-opacity="0.6" />
      <rect x="380" y="380" width="380" height="380" rx="84"
        :fill="'var(--logo-rect-3)'" fill-opacity="0.85" />
    </g>
  </svg>
</template>

<script setup lang="ts">
withDefaults(defineProps<{ size?: number }>(), { size: 28 })
</script>
