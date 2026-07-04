<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref } from "vue";
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
} from "../services/galleryService";
import type { GalleryView, McpServerStatus, Turn } from "../types/gallery";

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
  () => sessions.value.find((session) => session.id === selectedSessionId.value) ?? sessions.value[0],
);

// Local collapsed overrides — allow us to force-expand turns without backend toggle
const localTurnCollapsedOverride = ref<Record<string, boolean>>({});

function isTurnCollapsed(turn: Turn) {
  return localTurnCollapsedOverride.value[turn.id] ?? turn.collapsed;
}

function forceTurnExpanded(turnId: string) {
  localTurnCollapsedOverride.value = {
    ...localTurnCollapsedOverride.value,
    [turnId]: false,
  };
}

const selectedSessionForView = computed(() => {
  const session = selectedSession.value;
  if (!session) return null;
  return {
    ...session,
    turns: session.turns.map((turn) => ({
      ...turn,
      collapsed: isTurnCollapsed(turn),
    })),
  };
});

function replaceGalleryView(nextView: GalleryView) {
  galleryView.value = nextView;
  const nextSelectedId = nextView.selectedSessionId ?? nextView.sessions[0]?.id ?? "";
  if (!nextView.sessions.some((session) => session.id === selectedSessionId.value)) {
    selectedSessionId.value = nextSelectedId;
  }
}

async function selectSession(sessionId: string) {
  selectedSessionId.value = sessionId;
  selectedArtifactId.value = "";
  const view = await selectSessionCommand(sessionId);
  replaceGalleryView(view);
  await openLatestTurnAndScroll(sessionId);
}

async function changeSidebarMode(mode: "groups" | "projects") {
  replaceGalleryView(await setSidebarMode(mode));
}

async function toggleTurn(turnId: string) {
  if (!selectedSession.value) return;
  const currentTurn = selectedSession.value.turns.find((t) => t.id === turnId);
  const currentCollapsed = localTurnCollapsedOverride.value[turnId] ?? currentTurn?.collapsed ?? false;
  localTurnCollapsedOverride.value = {
    ...localTurnCollapsedOverride.value,
    [turnId]: !currentCollapsed,
  };
  replaceGalleryView(await toggleTurnCollapsed(selectedSession.value.id, turnId));
}

// ── Scroll-to-latest-turn ──

async function openLatestTurnAndScroll(sessionId: string) {
  const session = galleryView.value.sessions.find((s) => s.id === sessionId);
  if (!session || session.turns.length === 0) return;
  const lastTurn = session.turns[session.turns.length - 1];

  forceTurnExpanded(lastTurn.id);
  await waitForTurnDomReady(lastTurn.id);
  scrollTurnIntoView(lastTurn.id);

  // Secondary corrections after PDF rendering may change layout
  window.setTimeout(() => scrollTurnIntoView(lastTurn.id), 80);
  window.setTimeout(() => scrollTurnIntoView(lastTurn.id), 240);
}

async function waitForTurnDomReady(turnId: string) {
  await nextTick();
  await new Promise<void>((resolve) => {
    requestAnimationFrame(() => requestAnimationFrame(() => resolve()));
  });
  const target = document.getElementById(`turn-${turnId}`);
  if (target) return;
  for (let i = 0; i < 10; i++) {
    await new Promise((r) => window.setTimeout(r, 30));
    await nextTick();
    if (document.getElementById(`turn-${turnId}`)) return;
  }
}

function scrollTurnIntoView(turnId: string) {
  const container = document.querySelector<HTMLElement>(".content-scroll");
  const target = document.getElementById(`turn-${turnId}`);
  if (!container || !target) return;
  const containerRect = container.getBoundingClientRect();
  const targetRect = target.getBoundingClientRect();
  const delta = targetRect.top - containerRect.top;
  container.scrollTo({
    top: Math.max(0, container.scrollTop + delta - 8),
    behavior: "auto",
  });
}

function getLastTurnId(sessionId: string): string | null {
  const session = galleryView.value.sessions.find((s) => s.id === sessionId);
  if (!session || session.turns.length === 0) return null;
  return session.turns[session.turns.length - 1].id;
}

async function maybeScrollAfterGalleryUpdate(
  previousSessionId: string,
  previousLastTurnId: string | null,
) {
  const currentSessionId = selectedSessionId.value || previousSessionId;
  if (!currentSessionId) return;
  const currentLastTurnId = getLastTurnId(currentSessionId);
  if (!currentLastTurnId) return;
  if (currentLastTurnId !== previousLastTurnId) {
    await openLatestTurnAndScroll(currentSessionId);
  }
}

// ── Lifecycle ──

onMounted(async () => {
  const view = await listGalleryView();
  replaceGalleryView(view);
  const initialId = view.selectedSessionId ?? view.sessions[0]?.id ?? "";
  if (initialId) await openLatestTurnAndScroll(initialId);

  mcpStatus.value = await getMcpServerStatus();

  unlistenGallery = await listenGalleryUpdates(async (v) => {
    const prevId = selectedSessionId.value;
    const prevLast = getLastTurnId(prevId);
    replaceGalleryView(v);
    await maybeScrollAfterGalleryUpdate(prevId, prevLast);
  });

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
      @open-settings="settingsOpen = true"
    />
    <MainArea
      v-if="selectedSessionForView"
      :session="selectedSessionForView"
      :selected-artifact-id="selectedArtifactId"
      @toggle-turn="toggleTurn"
      @select-artifact="selectedArtifactId = $event"
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
