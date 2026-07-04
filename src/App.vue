<script setup lang="ts">
import GalleryPage from "./pages/GalleryPage.vue";

async function getWindow() {
  if (!("__TAURI_INTERNALS__" in window)) return null;
  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  return getCurrentWindow();
}
async function minimize() { (await getWindow())?.minimize(); }
async function toggleMaximize() { (await getWindow())?.toggleMaximize(); }
async function closeWindow() { (await getWindow())?.close(); }
</script>

<template>
  <div class="window-buttons">
    <button type="button" title="最小化" @click="minimize">−</button>
    <button type="button" title="最大化" @click="toggleMaximize">□</button>
    <button type="button" title="关闭" @click="closeWindow">×</button>
  </div>
  <GalleryPage />
</template>

<style>
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

* {
  box-sizing: border-box;
}

html, body, #app {
  width: 100%;
  height: 100%;
  margin: 0;
  overflow: hidden;
}

body {
  min-width: 960px;
}

button {
  font: inherit;
}

.window-buttons {
  position: fixed;
  top: 0;
  right: 0;
  z-index: 100;
  display: flex;
  height: 36px;
}

.window-buttons button {
  width: 42px;
  height: 100%;
  border: none;
  background: transparent;
  color: #a1a1aa;
  font-size: 13px;
  line-height: 1;
  display: grid;
  place-items: center;
  cursor: default;
}

.window-buttons button:hover {
  background: rgba(255, 255, 255, 0.08);
  color: #e4e4e7;
}

.window-buttons button:last-child:hover {
  background: #e81123;
  color: white;
}
</style>
