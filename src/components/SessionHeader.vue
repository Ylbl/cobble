<script setup lang="ts">
import type { Session } from "../types/gallery";

defineProps<{
  session: Session;
}>();

type AppWindow = {
  close: () => Promise<void>;
  minimize: () => Promise<void>;
  startDragging: () => Promise<void>;
  toggleMaximize: () => Promise<void>;
};

function isTauriRuntime() {
  return "__TAURI_INTERNALS__" in window;
}

async function getAppWindow(): Promise<AppWindow | null> {
  if (!isTauriRuntime()) {
    return null;
  }

  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  return getCurrentWindow();
}

async function minimizeWindow() {
  const appWindow = await getAppWindow();
  if (!appWindow) {
    return;
  }

  await appWindow.minimize();
}

async function toggleMaximizeWindow() {
  const appWindow = await getAppWindow();
  if (!appWindow) {
    return;
  }

  await appWindow.toggleMaximize();
}

async function closeWindow() {
  const appWindow = await getAppWindow();
  if (!appWindow) {
    return;
  }

  await appWindow.close();
}

async function startWindowDrag(event: PointerEvent) {
  if (event.button !== 0) {
    return;
  }

  const appWindow = await getAppWindow();
  if (!appWindow) {
    return;
  }

  await appWindow.startDragging();
}
</script>

<template>
  <header class="session-header" @pointerdown="startWindowDrag" @dblclick="toggleMaximizeWindow">
    <div class="title-box">
      <span class="session-title">{{ session.title }}</span>
      <span class="project-badge">▱ {{ session.projectName }}</span>
      <button class="more" type="button" title="更多" @pointerdown.stop @dblclick.stop>…</button>
    </div>
    <div class="window-actions" aria-label="Window actions">
      <button type="button" title="最小化" @pointerdown.stop @dblclick.stop @click="minimizeWindow">
        −
      </button>
      <button type="button" title="最大化" @pointerdown.stop @dblclick.stop @click="toggleMaximizeWindow">
        □
      </button>
      <button type="button" title="关闭" @pointerdown.stop @dblclick.stop @click="closeWindow">×</button>
    </div>
  </header>
</template>

<style scoped>
.session-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  min-width: 0;
  height: 44px;
  padding: 7px 12px 7px 8px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  background: #111111;
  cursor: default;
  user-select: none;
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

.project-badge {
  overflow: hidden;
  max-width: 170px;
  padding: 2px 6px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 4px;
  background: #222222;
  color: #b7b7bd;
  font-size: 11px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.more,
.window-actions button {
  border: 0;
  background: transparent;
  color: var(--muted);
  cursor: pointer;
}

.more {
  width: 20px;
  height: 20px;
  padding: 0;
}

.window-actions {
  display: flex;
  align-items: center;
  gap: 6px;
  color: var(--muted);
}

.window-actions button {
  display: grid;
  width: 24px;
  height: 24px;
  place-items: center;
  border-radius: 5px;
  font-size: 12px;
}

.window-actions button:hover {
  background: rgba(255, 255, 255, 0.07);
  color: #ffffff;
}

</style>
