<script setup lang="ts">
import GalleryPage from "./pages/GalleryPage.vue";
</script>

<template>
  <div class="app-shell">
    <header class="app-titlebar" data-tauri-drag-region @dblclick="toggleMaximize">
      <span class="titlebar-brand" data-tauri-drag-region>Cobble</span>
      <div class="titlebar-spacer" data-tauri-drag-region />
      <div class="window-controls">
        <button type="button" @click="minimize">−</button>
        <button type="button" @click="toggleMaximize">□</button>
        <button type="button" class="close" @click="close">×</button>
      </div>
    </header>
    <div class="app-body">
      <GalleryPage />
    </div>
  </div>
</template>

<script lang="ts">
let win: any = null;

async function getWin() {
  if (win) return win;
  if (!("__TAURI_INTERNALS__" in window)) return null;
  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  win = getCurrentWindow();
  return win;
}

async function minimize() {
  (await getWin())?.minimize();
}

async function toggleMaximize(e?: MouseEvent) {
  if (e) {
    const t = e.target as HTMLElement;
    if (t.closest(".window-controls")) return;
  }
  (await getWin())?.toggleMaximize();
}

async function close() {
  (await getWin())?.close();
}
</script>

<style>
html, body, #app {
  width: 100%;
  height: 100%;
  margin: 0;
  overflow: hidden;
}

body {
  min-width: 960px;
}

*, *::before, *::after {
  box-sizing: border-box;
}

:root {
  font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  color: #f4f4f5;
  background: #0f0f0f;
  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  --bg: #0f0f0f;
  --sidebar: #1b1b1b;
  --panel: #141414;
  --panel-soft: #1f1f1f;
  --panel-raised: #242424;
  --border: rgba(255, 255, 255, 0.12);
  --border-strong: rgba(255, 255, 255, 0.2);
  --text: #f4f4f5;
  --muted: #a1a1aa;
  --subtle: #71717a;
  --active: #333333;
  --accent: #f59e0b;
  --danger: #ef4444;
  --ok: #22c55e;
}

.app-shell {
  width: 100vw;
  height: 100vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--bg);
}

.app-titlebar {
  height: 36px;
  min-height: 36px;
  flex: 0 0 36px;
  display: flex;
  align-items: center;
  background: #111111;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  user-select: none;
  z-index: 100;
}

.titlebar-brand {
  font-size: 12px;
  font-weight: 600;
  color: #e8e8e8;
  padding-left: 10px;
  white-space: nowrap;
}

.titlebar-spacer {
  flex: 1;
  height: 100%;
}

.window-controls {
  height: 100%;
  display: flex;
  align-items: stretch;
  flex: 0 0 auto;
}

.window-controls button {
  width: 46px;
  height: 100%;
  border: none;
  background: transparent;
  color: #d4d4d4;
  font-size: 13px;
  line-height: 1;
  display: grid;
  place-items: center;
  cursor: default;
}

.window-controls button:hover {
  background: rgba(255, 255, 255, 0.1);
}

.window-controls button.close:hover {
  background: #e81123;
  color: white;
}

.app-body {
  flex: 1;
  min-height: 0;
  display: flex;
  overflow: hidden;
}
</style>
