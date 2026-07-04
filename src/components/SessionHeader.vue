<script setup lang="ts">
import type { Session } from "../types/gallery";

defineProps<{
  session: Session;
}>();
</script>

<template>
  <header class="session-header">
    <div class="title-box">
      <span class="session-title">{{ session.title }}</span>
    </div>
  </header>
</template>

<style scoped>
.session-header {
  display: flex;
  align-items: center;
  min-width: 0;
  height: 44px;
  padding: 7px 12px 7px 8px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  background: #111111;
}

.title-box {
  display: inline-flex;
  max-width: min(520px, 62vw);
  height: 28px;
  align-items: center;
  gap: 8px;
  padding: 0 8px;
  border: 1px solid rgba(255, 255, 255, 0.11);
  border-radius: 5px;
  background: #171717;
}

.session-title {
  overflow: hidden;
  color: #ffffff;
  font-size: 13px;
  font-weight: 650;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
