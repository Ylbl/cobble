<script setup lang="ts">
import { computed, inject, reactive, ref, watch } from "vue";
import SourceLogViewer from "./SourceLogViewer.vue";
import PdfJsViewer from "../pdf/PdfJsViewer.vue";
import { openPath } from "../../services/settingsService";
import type { Artifact, LayoutSize } from "../../types/gallery";
import { FLOW_WIDTH_KEY } from "./ArtifactFlowLayout.vue";

const props = defineProps<{
  artifact: Artifact;
  selected: boolean;
}>();

defineEmits<{
  select: [];
}>();

const tileRef = ref<HTMLElement | null>(null);

const menuOpen = ref(false);
const menuPosition = ref({ x: 0, y: 0 });
const viewerOpen = ref(false);

const pdfUrl = computed(() => props.artifact.pdfAssetUrl || props.artifact.pdfUrl || "");
const canShowPdf = computed(() => props.artifact.status === "finished" && Boolean(pdfUrl.value));
const canShowImage = computed(
  () => props.artifact.kind === "image" && Boolean(props.artifact.assetUrl || props.artifact.imageUrl),
);

const textTargets = computed(() => [
  { label: "源码", path: props.artifact.sourceFilePath },
  { label: "main.log", path: props.artifact.logFilePath },
  { label: "stdout", path: props.artifact.stdoutPath },
  { label: "stderr", path: props.artifact.stderrPath },
]);

const layoutSize = computed<LayoutSize>(() => {
  if (props.artifact.status === "failed") return "small";
  if (props.artifact.previewType === "small") return "small";
  switch (props.artifact.kind) {
    case "image":
      return "wide";
    case "pdf":
    case "latex":
      return "medium";
    case "svg":
      return "medium";
    default:
      return "medium";
  }
});

const hasPdfPath = computed(() => Boolean(props.artifact.pdfLocalFilePath));
const hasSourceOrLog = computed(
  () =>
    Boolean(props.artifact.sourceFilePath) ||
    Boolean(props.artifact.logFilePath) ||
    Boolean(props.artifact.stdoutPath) ||
    Boolean(props.artifact.stderrPath),
);
const hasAnyPath = computed(
  () =>
    Boolean(props.artifact.sourceFilePath) ||
    Boolean(props.artifact.pdfLocalFilePath) ||
    Boolean(props.artifact.localFilePath) ||
    Boolean(props.artifact.logFilePath),
);

// ---- Zoom (wheel scroll) ----
const BASE_WIDTHS: Record<LayoutSize, number> = {
  small: 180,
  medium: 320,
  wide: 480,
};

// Module-level store: per-artifact custom width (survives component remounts)
const artifactWidthMap = reactive<Record<string, number>>({});

const flowWidth = inject(FLOW_WIDTH_KEY, ref(600));

function getBaseWidth(size: LayoutSize): number {
  return BASE_WIDTHS[size] ?? 320;
}

function getCurrentWidth(): number {
  return artifactWidthMap[props.artifact.id] ?? getBaseWidth(layoutSize.value);
}

function setCurrentWidth(w: number) {
  artifactWidthMap[props.artifact.id] = w;
}

function getMaxWidthInRow(): number {
  const tile = tileRef.value;
  if (!tile) return getBaseWidth(layoutSize.value);

  const flowEl = tile.closest(".artifact-flow") as HTMLElement | null;
  if (!flowEl) return getBaseWidth(layoutSize.value);

  const flowRect = flowEl.getBoundingClientRect();
  const tileRect = tile.getBoundingClientRect();
  const flowStyle = window.getComputedStyle(flowEl);
  const paddingRight = parseFloat(flowStyle.paddingRight || "0");
  const contentRight = flowRect.right - paddingRight;
  const safetyGap = 4;

  return Math.max(getBaseWidth(layoutSize.value), contentRight - tileRect.left - safetyGap);
}

const tileWidth = computed(() => getCurrentWidth());

// Container resize: clamp down if tile now exceeds available space (never auto-expand)
watch(flowWidth, () => {
  const currentWidth = getCurrentWidth();
  const maxWidth = getMaxWidthInRow();
  const baseWidth = getBaseWidth(layoutSize.value);
  if (currentWidth > maxWidth) {
    setCurrentWidth(Math.max(baseWidth, maxWidth));
  }
});

function onWheel(event: WheelEvent) {
  event.preventDefault();
  const step = 40;
  const direction = event.deltaY < 0 ? 1 : -1;
  const baseWidth = getBaseWidth(layoutSize.value);
  const currentWidth = getCurrentWidth();
  const maxWidth = getMaxWidthInRow();
  const nextWidth = clamp(currentWidth + direction * step, baseWidth, Math.max(baseWidth, maxWidth));
  setCurrentWidth(nextWidth);
}

function clamp(value: number, min: number, max: number): number {
  return Math.max(min, Math.min(value, max));
}

// ---- Context menu ----
function openContextMenu(event: MouseEvent) {
  event.preventDefault();
  menuPosition.value = { x: event.clientX, y: event.clientY };
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

function showSourceLog() {
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
    ref="tileRef"
    class="artifact-tile"
    :class="[layoutSize, artifact.status, { selected }]"
    :style="{ width: tileWidth + 'px' }"
    tabindex="0"
    @click="$emit('select')"
    @keydown.enter="$emit('select')"
    @contextmenu.prevent="openContextMenu"
    @wheel.prevent="onWheel"
  >
    <div class="tile-surface">
      <!-- Image -->
      <img
        v-if="canShowImage"
        :src="artifact.assetUrl || artifact.imageUrl || ''"
        alt=""
        class="tile-image"
      />

      <!-- PDF / LaTeX (finished with PDF output) -->
      <PdfJsViewer
        v-else-if="canShowPdf"
        mode="inline"
        :pdf-url="pdfUrl"
        :artifact-id="artifact.id"
        :local-file-path="artifact.pdfLocalFilePath"
      />

      <!-- SVG inline -->
      <div
        v-else-if="artifact.kind === 'svg' && artifact.svg"
        class="tile-svg"
        v-html="artifact.svg"
      ></div>

      <!-- Failed -->
      <div v-else-if="artifact.status === 'failed'" class="tile-error">
        <span class="error-icon">✕</span>
        <span class="error-msg">{{ artifact.errorMessage || "编译失败" }}</span>
      </div>

      <!-- Finished but no preview -->
      <div v-else-if="artifact.status === 'finished'" class="tile-nopreview">
        <span class="nopreview-kind">{{ artifact.kind.toUpperCase() }}</span>
        <span class="nopreview-label">无预览</span>
      </div>

      <!-- Compiling / rendering / received placeholder -->
      <div v-else class="tile-placeholder">
        <span class="kind-badge">{{ artifact.kind.toUpperCase() }}</span>
        <span class="preview-line wide"></span>
        <span class="preview-line"></span>
        <span class="preview-line short"></span>
        <span
          v-if="artifact.status === 'compiling' || artifact.status === 'rendering'"
          class="state-pill"
        >
          编译中
        </span>
      </div>
    </div>

    <!-- Context menu (teleported) -->
    <Teleport to="body">
      <button
        v-if="menuOpen"
        class="context-backdrop"
        type="button"
        @click="closeContextMenu"
      ></button>
      <div
        v-if="menuOpen"
        class="context-menu"
        :style="{ left: `${menuPosition.x}px`, top: `${menuPosition.y}px` }"
        role="menu"
        @click.stop
      >
        <div class="menu-header">
          <span>{{ artifact.kind.toUpperCase() }}</span>
          <span :class="['menu-status', artifact.status]">{{ artifact.status }}</span>
        </div>
        <button
          v-if="hasPdfPath"
          type="button"
          role="menuitem"
          @click="openArtifactPath(artifact.pdfLocalFilePath)"
        >
          打开 PDF
        </button>
        <button
          v-if="hasSourceOrLog"
          type="button"
          role="menuitem"
          @click="showSourceLog"
        >
          查看源码 / 日志
        </button>
        <button
          v-if="hasAnyPath"
          type="button"
          role="menuitem"
          @click="openArtifactDirectory"
        >
          打开目录
        </button>
      </div>
    </Teleport>

    <!-- Source / Log viewer modal -->
    <SourceLogViewer
      v-if="viewerOpen"
      :title="artifact.title"
      :targets="textTargets"
      @close="viewerOpen = false"
    />
  </article>
</template>

<style scoped>
.artifact-tile {
  display: flex;
  flex-direction: column;
  flex: 0 0 auto;
  box-sizing: border-box;
  overflow: hidden;
  border: 1px solid rgba(255, 255, 255, 0.14);
  border-radius: 6px;
  background: #1c1c1c;
  cursor: pointer;
  outline: none;
  transition:
    border-color 140ms ease,
    background 140ms ease;
}

/* Width tiers — base widths controlled by JS zoom, CSS provides fallback */
.artifact-tile.small {
  width: 180px;
}

.artifact-tile.medium {
  width: 320px;
}

.artifact-tile.wide {
  width: 480px;
}

.artifact-tile:hover,
.artifact-tile:focus-visible {
  border-color: rgba(245, 158, 11, 0.55);
  background: #222222;
}

.artifact-tile.selected {
  border-color: var(--accent);
  box-shadow: inset 0 0 0 1px rgba(245, 158, 11, 0.4);
}

/* Surface (content area) — no footer, no padding, content fills it */
.tile-surface {
  position: relative;
  width: 100%;
  background: #151515;
}

/* Image */
.tile-image {
  display: block;
  width: 100%;
  height: auto;
  object-fit: contain;
  background: #ffffff;
}

/* SVG */
.tile-svg {
  width: 100%;
  padding: 12px;
  background: #fafafa;
}

.tile-svg :deep(svg) {
  display: block;
  max-width: 100%;
  height: auto;
}

/* Failed state */
.tile-error {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  min-height: 100px;
  padding: 16px;
  background:
    linear-gradient(135deg, rgba(248, 113, 113, 0.09), transparent 50%),
    #1a1515;
}

.error-icon {
  width: 36px;
  height: 36px;
  display: grid;
  place-items: center;
  border-radius: 50%;
  background: rgba(248, 113, 113, 0.18);
  color: #f87171;
  font-size: 16px;
  font-weight: 800;
}

.error-msg {
  color: #fca5a5;
  font-size: 12px;
  line-height: 1.4;
  text-align: center;
  word-break: break-word;
  max-width: 100%;
}

/* No preview state */
.tile-nopreview {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  min-height: 80px;
  padding: 12px;
  background: #1a1a1a;
}

.nopreview-kind {
  padding: 3px 8px;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.06);
  color: #a1a1aa;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.04em;
}

.nopreview-label {
  color: #71717a;
  font-size: 11px;
}

/* Placeholder */
.tile-placeholder {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 100px;
  background:
    linear-gradient(90deg, rgba(255, 255, 255, 0.04) 1px, transparent 1px),
    linear-gradient(rgba(255, 255, 255, 0.03) 1px, transparent 1px),
    #181818;
  background-size: 22px 22px;
}

.kind-badge {
  position: absolute;
  top: 10px;
  left: 10px;
  padding: 3px 8px;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.08);
  color: #d4d4d8;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.04em;
}

.preview-line {
  position: absolute;
  left: 12px;
  bottom: 24px;
  width: 46%;
  height: 6px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.16);
}

.preview-line.wide {
  bottom: 42px;
  width: 68%;
}

.preview-line.short {
  bottom: 10px;
  width: 28%;
}

.state-pill {
  position: absolute;
  right: 10px;
  top: 10px;
  padding: 4px 8px;
  border-radius: 999px;
  background: rgba(245, 158, 11, 0.2);
  color: #fbbf24;
  font-size: 11px;
  font-weight: 800;
}

/* Context menu backdrop */
.context-backdrop {
  position: fixed;
  inset: 0;
  z-index: 30;
  border: 0;
  background: transparent;
  cursor: default;
}

/* Context menu dropdown */
.context-menu {
  position: fixed;
  z-index: 31;
  display: grid;
  width: 168px;
  padding: 6px;
  border: 1px solid var(--border-strong);
  border-radius: 7px;
  background: #202020;
  box-shadow: 0 14px 34px rgba(0, 0, 0, 0.45);
}

.menu-header {
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

.menu-status.compiling,
.menu-status.rendering {
  color: #fbbf24;
}

.menu-status.failed {
  color: #f87171;
}

.menu-status.received {
  color: #a1a1aa;
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

.context-menu button:hover {
  background: rgba(255, 255, 255, 0.08);
  color: #ffffff;
}
</style>
