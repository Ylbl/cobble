<script setup lang="ts">
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
}>();
</script>

<template>
  <section class="turn-block">
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
  </section>
</template>

<style scoped>
.turn-block {
  margin-top: 13px;
}

.turn-body {
  padding: 8px 0;
}
</style>
