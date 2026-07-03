<script setup lang="ts">
import ArtifactCard from "./ArtifactCard.vue";
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
    <div v-if="!turn.collapsed" class="artifact-grid">
      <ArtifactCard
        v-for="artifact in turn.artifacts"
        :key="artifact.id"
        :artifact="artifact"
        :selected="artifact.id === selectedArtifactId"
        @select="$emit('select-artifact', artifact.id)"
      />
    </div>
  </section>
</template>

<style scoped>
.turn-block {
  margin-top: 13px;
}

.artifact-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(260px, 1fr));
  gap: 12px;
  padding: 12px 0 2px;
}

@media (max-width: 1120px) {
  .artifact-grid {
    grid-template-columns: 1fr;
  }
}
</style>
