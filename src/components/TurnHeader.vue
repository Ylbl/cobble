<script setup lang="ts">
import type { Turn } from "../types/gallery";

defineProps<{
  turn: Turn;
}>();

defineEmits<{
  toggle: [];
}>();
</script>

<template>
  <button class="turn-header" type="button" @click="$emit('toggle')">
    <span class="chevron">{{ turn.collapsed ? "▸" : "▾" }}</span>
    <span class="turn-title">第 {{ turn.index }} 次对话：{{ turn.hint }}</span>
    <span class="count">{{ turn.artifacts.length }} artifact</span>
  </button>
</template>

<style scoped>
.turn-header {
  display: grid;
  grid-template-columns: 22px minmax(0, 1fr) auto;
  align-items: center;
  width: 100%;
  min-height: 36px;
  padding: 0 12px;
  border: 1px solid rgba(245, 158, 11, 0.45);
  border-radius: 5px;
  background: #151515;
  color: var(--text);
  cursor: pointer;
  text-align: left;
}

.turn-header:hover {
  border-color: rgba(245, 158, 11, 0.8);
  background: #191919;
}

.chevron {
  color: var(--accent);
  font-size: 14px;
}

.turn-title {
  overflow: hidden;
  font-size: 13px;
  font-weight: 650;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.count {
  padding-left: 12px;
  color: var(--subtle);
  font-size: 11px;
}
</style>
