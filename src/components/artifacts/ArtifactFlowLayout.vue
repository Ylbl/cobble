<script lang="ts">
export const FLOW_WIDTH_KEY = Symbol("flowLayoutWidth");
</script>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, provide, ref } from "vue";

const containerRef = ref<HTMLElement | null>(null);
const flowWidth = ref(600);

let ro: ResizeObserver | null = null;

function updateWidth() {
  if (!containerRef.value) return;
  const rect = containerRef.value.getBoundingClientRect();
  flowWidth.value = rect.width;
}

onMounted(() => {
  updateWidth();
  if (containerRef.value) {
    ro = new ResizeObserver(() => updateWidth());
    ro.observe(containerRef.value);
  }
});

onBeforeUnmount(() => {
  ro?.disconnect();
  ro = null;
});

provide(FLOW_WIDTH_KEY, flowWidth);
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
