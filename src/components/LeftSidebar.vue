<script setup lang="ts">
import SidebarTopActions from "./SidebarTopActions.vue";
import SessionList from "./SessionList.vue";
import type { GalleryView, McpServerStatus, SidebarMode } from "../types/gallery";

defineProps<{
  galleryView: GalleryView;
  selectedSessionId: string;
  mcpStatus: McpServerStatus;
}>();

defineEmits<{
  "set-sidebar-mode": [mode: SidebarMode];
  "select-session": [sessionId: string];
  "delete-session": [sessionId: string];
  "open-settings": [];
}>();
</script>

<template>
  <aside class="left-sidebar">
    <SidebarTopActions />
    <SessionList
      :gallery-view="galleryView"
      :selected-session-id="selectedSessionId"
      @set-sidebar-mode="$emit('set-sidebar-mode', $event)"
      @select-session="$emit('select-session', $event)"
      @delete-session="$emit('delete-session', $event)"
    />
    <div class="sidebar-bottom">
      <div class="mcp-status" :title="mcpStatus.url ?? ''">
        <span class="status-dot" :class="mcpStatus.status"></span>
        <span class="status-label">MCP {{ mcpStatus.running ? 'Running' : mcpStatus.status === 'failed' ? 'Failed' : 'Stopped' }}</span>
      </div>
      <button class="icon-button" type="button" title="设置" @click="$emit('open-settings')">⚙</button>
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

.mcp-status {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: 8px;
  cursor: default;
}

.status-dot {
  width: 7px;
  height: 7px;
  border-radius: 999px;
  flex: 0 0 auto;
  background: var(--subtle);
}

.status-dot.running {
  background: var(--ok);
}

.status-dot.failed {
  background: var(--danger);
}

.status-dot.stopped {
  background: var(--subtle);
}

.status-label {
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
