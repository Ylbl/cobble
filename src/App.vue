<script setup lang="ts">
import GalleryPage from "./pages/GalleryPage.vue";
</script>

<template>
  <GalleryPage />
</template>

<style>
:root {
  font-family:
    Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI",
    sans-serif;
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

html,
body,
#app {
  width: 100%;
  height: 100%;
  margin: 0;
}

body {
  min-width: 960px;
  overflow: hidden;
}

button {
  font: inherit;
}
</style>
