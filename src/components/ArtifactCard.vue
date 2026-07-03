<script setup lang="ts">
import { computed, reactive, ref } from "vue";
import SourceLogViewer from "./artifacts/SourceLogViewer.vue";
import PdfJsViewer from "./pdf/PdfJsViewer.vue";
import { openPath } from "../services/settingsService";
import type { Artifact } from "../types/gallery";

const props = defineProps<{
  artifact: Artifact;
  selected: boolean;
}>();

defineEmits<{
  select: [];
}>();

const menuOpen = ref(false);
const viewerOpen = ref(false);
const menuPosition = reactive({ x: 0, y: 0 });

const pdfUrl = computed(() => props.artifact.pdfAssetUrl || props.artifact.pdfUrl || "");
const canShowPdf = computed(() => props.artifact.status === "finished" && Boolean(pdfUrl.value));
const textTargets = computed(() => [
  { label: "源码", path: props.artifact.sourceFilePath },
  { label: "main.log", path: props.artifact.logFilePath },
  { label: "stdout", path: props.artifact.stdoutPath },
  { label: "stderr", path: props.artifact.stderrPath },
]);

function openContextMenu(event: MouseEvent) {
  event.preventDefault();
  menuPosition.x = event.clientX;
  menuPosition.y = event.clientY;
  menuOpen.value = true;
}

function closeContextMenu() {
  menuOpen.value = false;
}

async function openArtifactPath(path?: string | null) {
  if (path) {
    await openPath(path);
  }
  closeContextMenu();
}

async function openArtifactDirectory() {
  const path =
    props.artifact.sourceFilePath ||
    props.artifact.pdfLocalFilePath ||
    props.artifact.localFilePath ||
    props.artifact.logFilePath;
  if (path) {
    await openPath(parentPath(path));
  }
  closeContextMenu();
}

function showTextViewer() {
  viewerOpen.value = true;
  closeContextMenu();
}

function parentPath(path: string) {
  const index = Math.max(path.lastIndexOf("\\"), path.lastIndexOf("/"));
  return index > 0 ? path.slice(0, index) : path;
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
        <img
          v-if="artifact.kind === 'image' && (artifact.assetUrl || artifact.imageUrl)"
          :src="artifact.assetUrl || artifact.imageUrl || ''"
          alt=""
        />
        <PdfJsViewer
          v-else-if="canShowPdf"
          :pdf-url="pdfUrl"
          :artifact-id="artifact.id"
          :local-file-path="artifact.pdfLocalFilePath"
          :initial-scale="0.85"
        />
        <template v-else>
          <span class="preview-kind">{{ artifact.kind.toUpperCase() }}</span>
          <span class="preview-line wide"></span>
          <span class="preview-line"></span>
          <span class="preview-line short"></span>
          <span v-if="artifact.kind === 'svg'" class="molecule"></span>
          <span v-else-if="artifact.kind === 'image'" class="image-mark"></span>
          <span v-if="artifact.status === 'compiling' || artifact.status === 'rendering'" class="state-pill">
            正在编译
          </span>
          <span v-if="artifact.status === 'failed'" class="state-pill failed">编译失败</span>
        </template>
      </div>
    </div>
    <div class="artifact-body">
      <h2>{{ artifact.title }}</h2>
      <p v-if="artifact.status === 'failed' && artifact.errorMessage">{{ artifact.errorMessage }}</p>
      <div class="artifact-actions">
        <button v-if="artifact.pdfLocalFilePath" type="button" @click.stop="openArtifactPath(artifact.pdfLocalFilePath)">
          打开 PDF
        </button>
        <button
          v-if="artifact.sourceFilePath || artifact.logFilePath || artifact.stdoutPath || artifact.stderrPath"
          type="button"
          @click.stop="showTextViewer"
        >
          查看源码 / 日志
        </button>
        <button type="button" @click.stop="openArtifactDirectory">打开目录</button>
      </div>
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
        <button type="button" role="menuitem" @click="openArtifactPath(props.artifact.pdfLocalFilePath || props.artifact.localFilePath)">
          打开文件
        </button>
        <button type="button" role="menuitem" @click="openArtifactDirectory">打开目录</button>
        <button type="button" role="menuitem" @click="showTextViewer">查看源码 / 日志</button>
      </div>
    </Teleport>
    <SourceLogViewer
      v-if="viewerOpen"
      :title="artifact.title"
      :targets="textTargets"
      @close="viewerOpen = false"
    />
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
  height: 236px;
  padding: 14px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  background:
    linear-gradient(90deg, rgba(255, 255, 255, 0.035) 1px, transparent 1px),
    linear-gradient(rgba(255, 255, 255, 0.03) 1px, transparent 1px), #101010;
  background-size: 22px 22px;
}

.small .preview {
  height: 172px;
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

.state-pill {
  position: absolute;
  right: 12px;
  top: 12px;
  padding: 4px 8px;
  border-radius: 999px;
  background: rgba(245, 158, 11, 0.16);
  color: #fbbf24;
  font-size: 11px;
  font-weight: 800;
}

.state-pill.failed {
  background: rgba(248, 113, 113, 0.15);
  color: #f87171;
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

p {
  margin: 6px 0 0;
  overflow: hidden;
  color: #f87171;
  font-size: 11px;
  line-height: 1.35;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.artifact-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 10px;
}

.artifact-actions button {
  height: 26px;
  border: 1px solid var(--border);
  border-radius: 5px;
  background: #202020;
  color: #d4d4d8;
  cursor: pointer;
  font-size: 12px;
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
  width: 168px;
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

.menu-status.rendering,
.menu-status.compiling {
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
