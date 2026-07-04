<script setup lang="ts">
import SessionHeader from "./SessionHeader.vue";
import TurnBlock from "./TurnBlock.vue";
import type { Session } from "../types/gallery";

defineProps<{
  session: Session;
  selectedArtifactId: string;
}>();

defineEmits<{
  "toggle-turn": [turnId: string];
  "select-artifact": [artifactId: string];
}>();
</script>

<template>
  <main class="main-area">
    <SessionHeader :session="session" />
    <section class="content-scroll">
      <TurnBlock
        v-for="turn in session.turns"
        :key="turn.id"
        :turn="turn"
        :selected-artifact-id="selectedArtifactId"
        @toggle="$emit('toggle-turn', turn.id)"
        @select-artifact="$emit('select-artifact', $event)"
      />
    </section>
  </main>
</template>

<style scoped>
.main-area {
  display: grid;
  grid-template-rows: 44px minmax(0, 1fr);
  min-width: 0;
  height: 100vh;
  background: var(--bg);
}

.content-scroll {
  min-height: 0;
  overflow-y: auto;
  padding: 16px 34px 28px;
}

.content-scroll::-webkit-scrollbar {
  width: 8px;
}

.content-scroll::-webkit-scrollbar-track {
  background: transparent;
}

.content-scroll::-webkit-scrollbar-thumb {
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.13);
}
</style>
