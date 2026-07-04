<script setup lang="ts">
import { ref } from "vue";
import type { Session } from "../types/gallery";

defineProps<{
  session: Session;
  active: boolean;
}>();

defineEmits<{
  click: [];
  "delete-session": [sessionId: string];
}>();

const menuOpen = ref(false);
const menuX = ref(0);
const menuY = ref(0);

function openMenu(e: MouseEvent) {
  e.preventDefault();
  menuX.value = Math.min(e.clientX, window.innerWidth - 150);
  menuY.value = Math.min(e.clientY, window.innerHeight - 60);
  menuOpen.value = true;
}

function closeMenu() {
  menuOpen.value = false;
}
</script>

<template>
  <button class="session-item" :class="{ active }" type="button" @click="$emit('click')" @contextmenu="openMenu">
    <span class="session-copy">
      <span class="title">{{ session.title }}</span>
    </span>
    <span class="updated">{{ session.updatedAtLabel }}</span>

    <Teleport to="body">
      <button v-if="menuOpen" class="menu-backdrop" type="button" @click="closeMenu" />
      <div v-if="menuOpen" class="context-menu" :style="{ left: menuX + 'px', top: menuY + 'px' }" @click.stop>
        <button class="danger" type="button" @click="$emit('delete-session', session.id); closeMenu()">删除会话</button>
      </div>
    </Teleport>
  </button>
</template>

<style scoped>
.session-item {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: center;
  width: 100%;
  min-height: 42px;
  gap: 8px;
  padding: 6px 7px;
  border: 1px solid transparent;
  border-radius: 7px;
  background: transparent;
  color: var(--muted);
  cursor: pointer;
  text-align: left;
}

.session-item:hover { background: rgba(255, 255, 255, 0.05); }
.session-item.active { background: var(--active); color: var(--text); }

.session-copy {
  display: grid;
  min-width: 0;
  gap: 1px;
}

.title {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: inherit;
  font-size: 12px;
  font-weight: 620;
}

.updated {
  color: var(--subtle);
  font-size: 11px;
  white-space: nowrap;
}
.active .updated { color: #c1c1c6; }

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
  width: 130px;
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
