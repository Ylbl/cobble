<script setup lang="ts">
import { ref } from "vue";
import ArtifactFlowLayout from "./artifacts/ArtifactFlowLayout.vue";
import ArtifactTile from "./artifacts/ArtifactTile.vue";
import TurnHeader from "./TurnHeader.vue";
import type { Turn } from "../types/gallery";

defineProps<{
  turn: Turn;
  selectedArtifactId: string;
}>();

defineEmits<{
  toggle: [];
  "select-artifact": [artifactId: string];
  "delete-turn": [turnId: string];
}>();

const turnMenuOpen = ref(false);
const turnMenuX = ref(0);
const turnMenuY = ref(0);

function openTurnMenu(e: MouseEvent) {
  const target = e.target as HTMLElement;
  // Don't show turn menu if clicking on an artifact tile
  if (target.closest(".artifact-tile")) return;
  e.preventDefault();
  turnMenuX.value = Math.min(e.clientX, window.innerWidth - 140);
  turnMenuY.value = Math.min(e.clientY, window.innerHeight - 40);
  turnMenuOpen.value = true;
}

function closeTurnMenu() {
  turnMenuOpen.value = false;
}
</script>

<template>
  <section class="turn-block" :id="`turn-${turn.id}`" @contextmenu="openTurnMenu">
    <TurnHeader :turn="turn" @toggle="$emit('toggle')" />
    <div v-if="!turn.collapsed" class="turn-body">
      <ArtifactFlowLayout>
        <ArtifactTile
          v-for="artifact in turn.artifacts"
          :key="artifact.id"
          :artifact="artifact"
          :selected="artifact.id === selectedArtifactId"
          @select="$emit('select-artifact', artifact.id)"
        />
      </ArtifactFlowLayout>
    </div>

    <Teleport to="body">
      <button v-if="turnMenuOpen" class="menu-backdrop" type="button" @click="closeTurnMenu" />
      <div v-if="turnMenuOpen" class="context-menu" :style="{ left: turnMenuX + 'px', top: turnMenuY + 'px' }" @click.stop>
        <button class="danger" type="button" @click="$emit('delete-turn', turn.id); closeTurnMenu()">删除本次对话</button>
      </div>
    </Teleport>
  </section>
</template>

<style scoped>
.turn-block { margin-top: 13px; }
.turn-body { padding: 8px 0; }

.menu-backdrop {
  position: fixed;
  inset: 0;
  z-index: 50;
  border: 0;
  background: transparent;
  cursor: default;
}

.context-menu {
  position: fixed;
  z-index: 51;
  display: grid;
  width: 140px;
  padding: 4px;
  border: 1px solid rgba(255, 255, 255, 0.14);
  border-radius: 6px;
  background: #1e1e1e;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
}

.context-menu button {
  height: 28px;
  border: 0;
  border-radius: 5px;
  background: transparent;
  color: #d4d4d8;
  cursor: pointer;
  font-size: 12px;
  text-align: left;
  padding: 0 8px;
}

.context-menu button:hover { background: rgba(255, 255, 255, 0.08); }
.context-menu button.danger { color: #f87171; }
.context-menu button.danger:hover { background: rgba(248, 113, 113, 0.12); }
</style>
