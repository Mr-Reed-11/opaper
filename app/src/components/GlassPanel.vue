<template>
  <div class="glass glass-shadow rounded-2xl" :class="$attrs.class">
    <slot />
  </div>
</template>

<script setup lang="ts">
defineOptions({ inheritAttrs: false })
</script>
