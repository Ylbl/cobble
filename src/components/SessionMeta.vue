<script setup lang="ts">
import type { Session } from "../types/gallery";

defineProps<{
  session: Session;
}>();
</script>

<template>
  <section class="session-meta">
    <h1>{{ session.title }}</h1>
  </section>
</template>

<style scoped>
.session-meta {
  margin-bottom: 18px;
  padding: 12px 14px;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.035), rgba(255, 255, 255, 0.015));
}

h1 {
  margin: 0;
  color: var(--text);
  font-size: 20px;
  line-height: 1.25;
}
</style>
