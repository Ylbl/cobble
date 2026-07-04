<script setup lang="ts">
import { computed } from "vue";
import SessionItem from "./SessionItem.vue";
import type { GalleryView, SidebarMode, Session } from "../types/gallery";

const props = defineProps<{
  galleryView: GalleryView;
  selectedSessionId: string;
}>();

defineEmits<{
  "set-sidebar-mode": [mode: SidebarMode];
  "select-session": [sessionId: string];
}>();

const sessionById = computed(() => new Map(props.galleryView.sessions.map((session) => [session.id, session])));

const sections = computed(() => {
  const source =
    props.galleryView.sidebarMode === "projects" ? props.galleryView.projects : props.galleryView.groups;
  return source.map((section) => ({
    id: section.id,
    name: section.name,
    sessions: section.sessionIds
      .map((sessionId) => sessionById.value.get(sessionId))
      .filter((session): session is Session => Boolean(session)),
  }));
});
</script>

<template>
  <div class="session-list-wrap">
    <div class="segmented">
      <button
        :class="{ active: galleryView.sidebarMode === 'groups', muted: galleryView.sidebarMode !== 'groups' }"
        type="button"
        @click="$emit('set-sidebar-mode', 'groups')"
      >
        分组
      </button>
      <button
        :class="{ active: galleryView.sidebarMode === 'projects', muted: galleryView.sidebarMode !== 'projects' }"
        type="button"
        @click="$emit('set-sidebar-mode', 'projects')"
      >
        项目
      </button>
      <button class="muted icon" type="button" title="过滤">≡</button>
    </div>
    <nav class="session-list" aria-label="Sessions">
      <template v-for="section in sections" :key="section.id">
        <div class="group-title">
          <span>▿ {{ section.name }}</span>
        </div>
        <SessionItem
          v-for="session in section.sessions"
          :key="session.id"
          :session="session"
          :active="session.id === selectedSessionId"
          @click="$emit('select-session', session.id)"
        />
      </template>
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
</style>
