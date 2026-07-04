<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import LeftSidebar from "../components/LeftSidebar.vue";
import MainArea from "../components/MainArea.vue";
import SettingsPage from "./SettingsPage.vue";
import { getMcpServerStatus, listenMcpStatusUpdates } from "../services/mcpStatusService";
import {
  listGalleryView,
  listenGalleryUpdates,
  selectSession as selectSessionCommand,
  setSidebarMode,
  toggleTurnCollapsed,
  deleteGallerySession,
  deleteGalleryTurn,
} from "../services/galleryService";
import type { GalleryView, McpServerStatus } from "../types/gallery";

const galleryView = ref<GalleryView>({
  sidebarMode: "groups",
  groups: [],
  projects: [],
  sessions: [],
  selectedSessionId: null,
});
const selectedSessionId = ref("");
const selectedArtifactId = ref("");
const settingsOpen = ref(false);
const mcpStatus = ref<McpServerStatus>({
  running: false,
  url: null,
  port: null,
});
let unlistenGallery: (() => void) | undefined;
let unlistenMcpStatus: (() => void) | undefined;

const sessions = computed(() => galleryView.value.sessions);
const selectedSession = computed(
  () => sessions.value.find((s) => s.id === selectedSessionId.value) ?? sessions.value[0],
);

function replaceGalleryView(nextView: GalleryView) {
  galleryView.value = nextView;
  const nextSelectedId = nextView.selectedSessionId ?? nextView.sessions[0]?.id ?? "";
  if (!nextView.sessions.some((s) => s.id === selectedSessionId.value)) {
    selectedSessionId.value = nextSelectedId;
  }
}

async function selectSession(sessionId: string) {
  selectedSessionId.value = sessionId;
  selectedArtifactId.value = "";
  replaceGalleryView(await selectSessionCommand(sessionId));
}

async function changeSidebarMode(mode: "groups" | "projects") {
  replaceGalleryView(await setSidebarMode(mode));
}

async function toggleTurn(turnId: string) {
  if (selectedSession.value) {
    replaceGalleryView(await toggleTurnCollapsed(selectedSession.value.id, turnId));
  }
}

async function deleteSession(sessionId: string) {
  replaceGalleryView(await deleteGallerySession(sessionId));
}

async function deleteTurn(turnId: string) {
  if (selectedSession.value) {
    replaceGalleryView(await deleteGalleryTurn(selectedSession.value.id, turnId));
  }
}

onMounted(async () => {
  replaceGalleryView(await listGalleryView());
  mcpStatus.value = await getMcpServerStatus();
  unlistenGallery = await listenGalleryUpdates(replaceGalleryView);
  unlistenMcpStatus = await listenMcpStatusUpdates((status) => {
    mcpStatus.value = status;
  });
});

onUnmounted(() => {
  unlistenGallery?.();
  unlistenMcpStatus?.();
});
</script>

<template>
  <div class="gallery-shell">
    <LeftSidebar
      :gallery-view="galleryView"
      :selected-session-id="selectedSessionId"
      :mcp-status="mcpStatus"
      @set-sidebar-mode="changeSidebarMode"
      @select-session="selectSession"
      @delete-session="deleteSession"
      @open-settings="settingsOpen = true"
    />
    <MainArea
      v-if="selectedSession"
      :session="selectedSession"
      :selected-artifact-id="selectedArtifactId"
      @toggle-turn="toggleTurn"
      @select-artifact="selectedArtifactId = $event"
      @delete-turn="deleteTurn"
    />
    <main v-else class="empty-main">
      <div class="empty-state">暂无 artifact session</div>
    </main>
    <SettingsPage
      v-if="settingsOpen"
      :mcp-status="mcpStatus"
      @close="settingsOpen = false"
      @mcp-status="mcpStatus = $event"
      @gallery-view="replaceGalleryView"
    />
  </div>
</template>

<style scoped>
.gallery-shell {
  display: grid;
  grid-template-columns: 240px minmax(0, 1fr);
  width: 100vw;
  height: 100vh;
  overflow: hidden;
  background:
    radial-gradient(circle at 72% 0%, rgba(245, 158, 11, 0.04), transparent 32%),
    var(--bg);
}

.empty-main {
  display: grid;
  height: 100vh;
  place-items: center;
  border-left: 1px solid var(--border);
  background: var(--bg);
  color: var(--muted);
}

.empty-state {
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 14px 18px;
  background: var(--panel);
}
</style>
