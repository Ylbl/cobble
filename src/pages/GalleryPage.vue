<script setup lang="ts">
import { computed, reactive, ref } from "vue";
import LeftSidebar from "../components/LeftSidebar.vue";
import MainArea from "../components/MainArea.vue";
import { mockSessions } from "../data/mockSessions";
import type { Session } from "../types/gallery";

const sessions = reactive<Session[]>(
  mockSessions.map((session) => ({
    ...session,
    turns: session.turns.map((turn) => ({ ...turn, artifacts: [...turn.artifacts] })),
  })),
);

const selectedSessionId = ref(sessions[0]?.id ?? "");
const selectedArtifactId = ref("");

const selectedSession = computed(
  () => sessions.find((session) => session.id === selectedSessionId.value) ?? sessions[0],
);

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
      :selected-artifact-id="selectedArtifactId"
      @toggle-turn="toggleTurn"
      @select-artifact="selectedArtifactId = $event"
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
</style>
