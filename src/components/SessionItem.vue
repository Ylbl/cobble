<script setup lang="ts">
import type { Session } from "../types/gallery";

defineProps<{
  session: Session;
  active: boolean;
}>();

defineEmits<{
  click: [];
}>();
</script>

<template>
  <button class="session-item" :class="{ active }" type="button" @click="$emit('click')">
    <span class="status-dot" :class="(session.clientName ?? 'Unknown').toLowerCase()"></span>
    <span class="session-copy">
      <span class="title">{{ session.title }}</span>
    </span>
    <span class="updated">{{ session.updatedAtLabel }}</span>
  </button>
</template>

<style scoped>
.session-item {
  display: grid;
  grid-template-columns: 10px minmax(0, 1fr) auto;
  align-items: center;
  width: 100%;
  min-height: 42px;
  gap: 8px;
  padding: 6px 7px;
  border: 1px solid transparent;
  border-radius: 7px;
  background: transparent;
  color: var(--muted);
  cursor: pointer;
  text-align: left;
}

.session-item:hover {
  background: rgba(255, 255, 255, 0.05);
}

.session-item.active {
  background: var(--active);
  color: var(--text);
}

.status-dot {
  width: 5px;
  height: 5px;
  border-radius: 50%;
  background: var(--subtle);
}

.status-dot.zcode {
  background: var(--accent);
}

.status-dot.codex {
  background: #38bdf8;
}

.status-dot.cursor {
  background: #a78bfa;
}

.status-dot.unknown {
  background: var(--subtle);
}

.session-copy {
  display: grid;
  min-width: 0;
  gap: 1px;
}

.title {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.title {
  color: inherit;
  font-size: 12px;
  font-weight: 620;
}

.updated {
  color: var(--subtle);
  font-size: 11px;
  white-space: nowrap;
}

.active .updated {
  color: #c1c1c6;
}
</style>
