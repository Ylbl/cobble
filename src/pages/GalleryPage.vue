<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import LeftSidebar from "../components/LeftSidebar.vue";
import MainArea from "../components/MainArea.vue";
import { getMcpServerStatus } from "../services/mcpStatusService";
import { listGalleryView, listenGalleryUpdates } from "../services/galleryService";
import type { GalleryView, McpServerStatus } from "../types/gallery";

const galleryView = ref<GalleryView>({
  sessions: [],
  selectedSessionId: null,
});
const selectedSessionId = ref("");
const selectedArtifactId = ref("");
const mcpStatus = ref<McpServerStatus>({
  running: false,
  url: null,
  port: null,
});
let unlistenGallery: (() => void) | undefined;

const sessions = computed(() => galleryView.value.sessions);

const selectedSession = computed(
  () => sessions.value.find((session) => session.id === selectedSessionId.value) ?? sessions.value[0],
);

function replaceGalleryView(nextView: GalleryView) {
  galleryView.value = nextView;
  const nextSelectedId = nextView.selectedSessionId ?? nextView.sessions[0]?.id ?? "";
  if (!nextView.sessions.some((session) => session.id === selectedSessionId.value)) {
    selectedSessionId.value = nextSelectedId;
  }
}

function selectSession(sessionId: string) {
  selectedSessionId.value = sessionId;
  selectedArtifactId.value = "";
}

function toggleTurn(turnId: string) {
  const turn = selectedSession.value?.turns.find((item) => item.id === turnId);
  if (turn) {
    turn.collapsed = !turn.collapsed;
  }
}

onMounted(async () => {
  replaceGalleryView(await listGalleryView());
  mcpStatus.value = await getMcpServerStatus();
  unlistenGallery = await listenGalleryUpdates(replaceGalleryView);
});

onUnmounted(() => {
  unlistenGallery?.();
});
</script>

<template>
  <div class="gallery-shell">
    <LeftSidebar
      :sessions="sessions"
      :selected-session-id="selectedSessionId"
      @select-session="selectSession"
    />
    <MainArea
      v-if="selectedSession"
      :session="selectedSession"
      :mcp-status="mcpStatus"
      :selected-artifact-id="selectedArtifactId"
      @toggle-turn="toggleTurn"
      @select-artifact="selectedArtifactId = $event"
    />
    <main v-else class="empty-main">
      <div class="empty-state">暂无 artifact session</div>
    </main>
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
