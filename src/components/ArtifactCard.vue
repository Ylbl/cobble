<script setup lang="ts">
import { reactive, ref } from "vue";
import type { Artifact } from "../types/gallery";

defineProps<{
  artifact: Artifact;
  selected: boolean;
}>();

defineEmits<{
  select: [];
}>();

const menuOpen = ref(false);
const menuPosition = reactive({ x: 0, y: 0 });

function openContextMenu(event: MouseEvent) {
  event.preventDefault();
  menuPosition.x = event.clientX;
  menuPosition.y = event.clientY;
  menuOpen.value = true;
}

function closeContextMenu() {
  menuOpen.value = false;
}
</script>

<template>
  <article
    class="artifact-card"
    :class="[artifact.previewType, artifact.status, { selected }]"
    tabindex="0"
    @click="$emit('select')"
    @contextmenu="openContextMenu"
    @keydown.enter="$emit('select')"
  >
    <div class="preview">
      <div class="preview-sheet">
        <span class="preview-kind">{{ artifact.kind.toUpperCase() }}</span>
        <span class="preview-line wide"></span>
        <span class="preview-line"></span>
        <span class="preview-line short"></span>
        <span v-if="artifact.kind === 'svg'" class="molecule"></span>
        <img v-if="artifact.kind === 'image' && artifact.imageUrl" :src="artifact.imageUrl" alt="" />
        <span v-else-if="artifact.kind === 'image'" class="image-mark"></span>
      </div>
    </div>
    <div class="artifact-body">
      <h2>{{ artifact.title }}</h2>
    </div>
    <Teleport to="body">
      <button v-if="menuOpen" class="context-backdrop" type="button" @click="closeContextMenu"></button>
      <div
        v-if="menuOpen"
        class="artifact-menu"
        :style="{ left: `${menuPosition.x}px`, top: `${menuPosition.y}px` }"
        role="menu"
        @click.stop
      >
        <div class="menu-meta">
          <span>{{ artifact.kind.toUpperCase() }}</span>
          <span :class="['menu-status', artifact.status]">{{ artifact.status }}</span>
        </div>
        <button type="button" role="menuitem" @click="closeContextMenu">打开</button>
        <button type="button" role="menuitem" @click="closeContextMenu">复制路径</button>
        <button type="button" role="menuitem" @click="closeContextMenu">查看源码</button>
        <button type="button" role="menuitem" @click="closeContextMenu">查看日志</button>
      </div>
    </Teleport>
  </article>
</template>

<style scoped>
.artifact-card {
  min-width: 0;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--panel);
  cursor: pointer;
  outline: none;
  transition:
    border-color 140ms ease,
    background 140ms ease,
    transform 140ms ease;
}

.artifact-card:hover,
.artifact-card:focus-visible {
  border-color: rgba(245, 158, 11, 0.5);
  background: #181818;
}

.artifact-card.selected {
  border-color: var(--accent);
  box-shadow: inset 0 0 0 1px rgba(245, 158, 11, 0.34);
}

.artifact-card.large {
  grid-column: span 2;
}

.preview {
  height: 174px;
  padding: 14px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  background:
    linear-gradient(90deg, rgba(255, 255, 255, 0.035) 1px, transparent 1px),
    linear-gradient(rgba(255, 255, 255, 0.03) 1px, transparent 1px), #101010;
  background-size: 22px 22px;
}

.small .preview {
  height: 132px;
}

.preview-sheet {
  position: relative;
  width: 100%;
  height: 100%;
  overflow: hidden;
  border: 1px solid rgba(255, 255, 255, 0.16);
  border-radius: 6px;
  background:
    linear-gradient(135deg, rgba(245, 158, 11, 0.08), transparent 46%),
    linear-gradient(180deg, #232323, #171717);
}

.preview-sheet img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.preview-kind {
  position: absolute;
  top: 12px;
  left: 12px;
  color: #ffffff;
  font-size: 11px;
  font-weight: 800;
}

.preview-line {
  position: absolute;
  left: 12px;
  bottom: 24px;
  width: 46%;
  height: 5px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.13);
}

.preview-line.wide {
  bottom: 42px;
  width: 68%;
}

.preview-line.short {
  bottom: 11px;
  width: 28%;
}

.molecule,
.image-mark {
  position: absolute;
  right: 22px;
  bottom: 20px;
  width: 72px;
  height: 48px;
}

.molecule {
  border: 2px solid rgba(245, 158, 11, 0.75);
  clip-path: polygon(25% 0%, 75% 0%, 100% 50%, 75% 100%, 25% 100%, 0% 50%);
}

.image-mark {
  border-radius: 7px;
  background:
    linear-gradient(135deg, transparent 48%, rgba(56, 189, 248, 0.8) 49% 52%, transparent 53%),
    radial-gradient(circle at 25% 28%, rgba(245, 158, 11, 0.9) 0 9px, transparent 10px),
    rgba(255, 255, 255, 0.08);
}

.artifact-body {
  padding: 12px;
}

h2 {
  min-width: 0;
  margin: 0;
  overflow: hidden;
  color: var(--text);
  font-size: 13px;
  line-height: 1.35;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.context-backdrop {
  position: fixed;
  inset: 0;
  z-index: 20;
  border: 0;
  background: transparent;
  cursor: default;
}

.artifact-menu {
  position: fixed;
  z-index: 21;
  display: grid;
  width: 148px;
  padding: 6px;
  border: 1px solid var(--border-strong);
  border-radius: 7px;
  background: #202020;
  box-shadow: 0 14px 34px rgba(0, 0, 0, 0.45);
}

.menu-meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 5px 7px 7px;
  color: var(--subtle);
  font-size: 10px;
  font-weight: 750;
}

.menu-status.finished {
  color: #86efac;
}

.menu-status.rendering {
  color: #fbbf24;
}

.menu-status.failed {
  color: #f87171;
}

.artifact-menu button {
  height: 28px;
  border: 0;
  border-radius: 5px;
  background: transparent;
  color: #d4d4d8;
  cursor: pointer;
  font-size: 12px;
  text-align: left;
}

.artifact-menu button:hover {
  background: rgba(255, 255, 255, 0.08);
  color: #ffffff;
}

@media (max-width: 1120px) {
  .artifact-card.large {
    grid-column: span 1;
  }
}
</style>
