<script setup lang="ts">
import SessionItem from "./SessionItem.vue";
import type { Session } from "../types/gallery";

defineProps<{
  sessions: Session[];
  selectedSessionId: string;
}>();

defineEmits<{
  "select-session": [sessionId: string];
}>();
</script>

<template>
  <div class="session-list-wrap">
    <div class="segmented">
      <button class="muted" type="button">分组</button>
      <button class="active" type="button">项目</button>
      <button class="muted icon" type="button" title="过滤">≡</button>
    </div>
    <div class="group-title">
      <span>▿ Desktop</span>
    </div>
    <nav class="session-list" aria-label="Sessions">
      <SessionItem
        v-for="session in sessions"
        :key="session.id"
        :session="session"
        :active="session.id === selectedSessionId"
        @click="$emit('select-session', session.id)"
      />
      <button class="show-more" type="button">显示更多</button>
    </nav>
  </div>
</template>

<style scoped>
.session-list-wrap {
  min-height: 0;
  padding: 9px 8px;
  overflow: hidden;
}

.segmented {
  display: flex;
  align-items: center;
  gap: 4px;
  height: 24px;
  margin-bottom: 8px;
}

.segmented button {
  height: 22px;
  border: 1px solid transparent;
  border-radius: 6px;
  background: transparent;
  color: var(--subtle);
  cursor: pointer;
  font-size: 11px;
}

.segmented .active {
  border-color: var(--border);
  background: #111111;
  color: var(--text);
}

.segmented .icon {
  margin-left: auto;
  width: 22px;
}

.group-title {
  display: flex;
  align-items: center;
  height: 24px;
  padding: 0 7px;
  color: var(--subtle);
  font-size: 12px;
}

.session-list {
  height: calc(100% - 56px);
  overflow-y: auto;
  padding-right: 2px;
}

.session-list::-webkit-scrollbar {
  width: 6px;
}

.session-list::-webkit-scrollbar-thumb {
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.14);
}

.show-more {
  width: 100%;
  height: 27px;
  padding-left: 24px;
  border: 0;
  background: transparent;
  color: #6f6f76;
  cursor: pointer;
  font-size: 12px;
  text-align: left;
}
</style>
