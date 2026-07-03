<script setup lang="ts">
import { onBeforeUnmount, ref } from "vue";

const containerRef = ref<HTMLElement | null>(null);
let ro: ResizeObserver | null = null;

function setupObserver() {
  if (!containerRef.value) return;
  ro = new ResizeObserver((entries) => {
    for (const entry of entries) {
      const width = entry.contentRect.width;
      containerRef.value?.style.setProperty("--flow-layout-width", `${width}px`);
    }
  });
  ro.observe(containerRef.value);
}

setupObserver();

onBeforeUnmount(() => {
  ro?.disconnect();
  ro = null;
});
</script>

<template>
  <div ref="containerRef" class="artifact-flow">
    <slot />
  </div>
</template>

<style scoped>
.artifact-flow {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-start;
  align-content: flex-start;
  gap: 16px;
  padding: 4px 0;
}
</style>
