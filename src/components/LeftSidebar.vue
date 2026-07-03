<script setup lang="ts">
import SidebarTopActions from "./SidebarTopActions.vue";
import SessionList from "./SessionList.vue";
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
  <aside class="left-sidebar">
    <SidebarTopActions />
    <SessionList
      :sessions="sessions"
      :selected-session-id="selectedSessionId"
      @select-session="$emit('select-session', $event)"
    />
    <div class="sidebar-bottom">
      <div class="user-badge" aria-label="Current user">
        <span class="avatar">O</span>
        <span class="user-name">ocmvnmqz</span>
      </div>
      <button class="icon-button" type="button" title="设置">⚙</button>
    </div>
  </aside>
</template>

<style scoped>
.left-sidebar {
  display: grid;
  grid-template-rows: auto minmax(0, 1fr) auto;
  width: 240px;
  height: 100vh;
  background: var(--sidebar);
  border-right: 1px solid var(--border);
  color: var(--text);
}

.sidebar-bottom {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 11px 12px 12px;
  border-top: 1px solid rgba(255, 255, 255, 0.08);
}

.user-badge {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: 9px;
}

.avatar {
  display: grid;
  width: 24px;
  height: 24px;
  place-items: center;
  border-radius: 50%;
  background: #111111;
  color: var(--text);
  font-size: 12px;
  font-weight: 700;
}

.user-name {
  overflow: hidden;
  color: #d4d4d8;
  font-size: 12px;
  font-weight: 650;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.icon-button {
  width: 26px;
  height: 26px;
  border: 1px solid transparent;
  border-radius: 6px;
  background: transparent;
  color: var(--muted);
  cursor: pointer;
}

.icon-button:hover {
  border-color: var(--border);
  background: rgba(255, 255, 255, 0.06);
  color: var(--text);
}
</style>
