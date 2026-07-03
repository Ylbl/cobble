<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref, shallowRef, watch } from "vue";
import * as pdfjsLib from "pdfjs-dist";
import pdfWorkerUrl from "pdfjs-dist/build/pdf.worker.mjs?url";
import type { PdfAssetDiagnostic } from "../../services/fileService";

const props = withDefaults(
  defineProps<{
    pdfUrl: string;
    artifactId?: string;
    localFilePath?: string | null;
    initialScale?: number;
    mode?: "inline" | "full";
    renderAllPages?: boolean;
  }>(),
  {
    initialScale: 1,
    mode: "full",
    renderAllPages: true,
  },
);

pdfjsLib.GlobalWorkerOptions.workerSrc = pdfWorkerUrl;

// ---- Refs ----
const containerRef = ref<HTMLDivElement | null>(null);
const canvasRef = ref<HTMLCanvasElement | null>(null);
const pagesContainerRef = ref<HTMLDivElement | null>(null);

const pdfDoc = shallowRef<any>(null);
const renderTask = shallowRef<any>(null);
const currentPage = ref(1);
const pageCount = ref(0);
const scale = ref(props.initialScale);
const loading = ref(false);
const errorMessage = ref("");
const diagnostic = ref<PdfAssetDiagnostic | null>(null);
const diagnosticError = ref("");
const fetchProbeStatus = ref("");
const loadSource = ref("");

// ---- Observers ----
let io: IntersectionObserver | null = null;
let ro: ResizeObserver | null = null;
let resizeTimer: ReturnType<typeof setTimeout> | null = null;
const isVisible = ref(props.mode === "full");
const isInline = () => props.mode === "inline";

const pageRenderTasks: any[] = [];

// ---- Cleanup ----
function cleanupRenderTask() {
  if (renderTask.value) {
    renderTask.value.cancel();
    renderTask.value = null;
  }
}

function cleanupAllPageTasks() {
  for (const task of pageRenderTasks) {
    if (task) task.cancel();
  }
  pageRenderTasks.length = 0;
}

function clearPagesContainer() {
  if (pagesContainerRef.value) {
    pagesContainerRef.value.innerHTML = "";
  }
}

function resetPdfState() {
  cleanupRenderTask();
  cleanupAllPageTasks();
  clearPagesContainer();
  pdfDoc.value?.destroy?.();
  pdfDoc.value = null;
  currentPage.value = 1;
  pageCount.value = 0;
  errorMessage.value = "";
  diagnostic.value = null;
  diagnosticError.value = "";
  fetchProbeStatus.value = "";
  loadSource.value = "";
}

// ---- Watchers ----
watch(
  () => props.pdfUrl,
  async () => {
    if (isInline()) return;
    await loadPdf();
  },
  { immediate: true },
);

watch(
  () => props.pdfUrl,
  async () => {
    if (!isInline() || !isVisible.value) return;
    await loadPdf();
  },
);

watch([currentPage, scale], async () => {
  if (isInline()) return;
  await renderPage();
});

// ---- IntersectionObserver (inline mode) ----
function setupIntersectionObserver() {
  if (!isInline() || !containerRef.value) return;
  io = new IntersectionObserver(
    (entries) => {
      const entry = entries[0];
      if (!entry) return;
      if (entry.isIntersecting) {
        if (!isVisible.value) {
          isVisible.value = true;
          loadPdf();
        }
        io?.disconnect();
        io = null;
      }
    },
    { rootMargin: "200px" },
  );
  io.observe(containerRef.value);
}

// ---- ResizeObserver (inline mode) ----
function setupResizeObserver() {
  if (!isInline() || !containerRef.value) return;
  ro = new ResizeObserver(() => {
    if (resizeTimer) clearTimeout(resizeTimer);
    resizeTimer = setTimeout(() => {
      if (pdfDoc.value && pageCount.value > 0) {
        renderAllPagesInline();
      }
    }, 150);
  });
  ro.observe(containerRef.value);
}

onMounted(() => {
  setupIntersectionObserver();
  setupResizeObserver();
});

onBeforeUnmount(() => {
  io?.disconnect();
  io = null;
  ro?.disconnect();
  ro = null;
  if (resizeTimer) clearTimeout(resizeTimer);
  resetPdfState();
});

// ---- Load PDF ----
async function loadPdf() {
  resetPdfState();
  if (!props.pdfUrl) return;

  loading.value = true;
  try {
    if (props.artifactId) {
      const { diagnosePdfAssetPath, readBinaryFile, recordPdfjsPreviewRequested } = await import(
        "../../services/fileService"
      );
      try {
        await recordPdfjsPreviewRequested(props.artifactId, props.pdfUrl);
      } catch (error) {
        diagnosticError.value = `record failed: ${formatError(error)}`;
      }
      if (props.localFilePath) {
        try {
          diagnostic.value = await diagnosePdfAssetPath(
            props.artifactId,
            props.localFilePath,
            props.pdfUrl,
          );
        } catch (error) {
          diagnosticError.value = `diagnose failed: ${formatError(error)}`;
        }
        try {
          const pdfBytes = await readBinaryFile(props.localFilePath);
          pdfDoc.value = await pdfjsLib.getDocument({ data: pdfBytes }).promise;
          loadSource.value = "local-bytes";
          pageCount.value = pdfDoc.value.numPages;
          await nextTick();
          if (isInline()) {
            await renderAllPagesInline();
          } else {
            await renderPage();
          }
          return;
        } catch (error) {
          diagnosticError.value = `local bytes failed: ${formatError(error)}`;
        }
      }
    }
    await probePdfUrl();
    pdfDoc.value = await pdfjsLib.getDocument({ url: props.pdfUrl }).promise;
    loadSource.value = "asset-url";
    pageCount.value = pdfDoc.value.numPages;
    await nextTick();
    if (isInline()) {
      await renderAllPagesInline();
    } else {
      await renderPage();
    }
  } catch (error) {
    console.error("PDF.js failed to load PDF", error);
    errorMessage.value = error instanceof Error ? error.message : "PDF 加载失败";
  } finally {
    loading.value = false;
  }
}

function formatError(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

async function probePdfUrl() {
  try {
    const response = await fetch(props.pdfUrl, { method: "GET", cache: "no-store" });
    fetchProbeStatus.value = `${response.status} ${response.statusText || ""}`.trim();
  } catch (error) {
    fetchProbeStatus.value = error instanceof Error ? error.message : "fetch probe failed";
  }
}

// ---- Single page render (full mode) ----
async function renderPage() {
  if (!pdfDoc.value || !canvasRef.value) return;
  cleanupRenderTask();
  try {
    const page = await pdfDoc.value.getPage(currentPage.value);
    const viewport = page.getViewport({ scale: scale.value });
    const canvas = canvasRef.value;
    const context = canvas.getContext("2d");
    if (!context) return;
    const dpr = window.devicePixelRatio || 1;
    canvas.width = Math.floor(viewport.width * dpr);
    canvas.height = Math.floor(viewport.height * dpr);
    canvas.style.width = `${viewport.width}px`;
    canvas.style.height = `${viewport.height}px`;
    context.setTransform(dpr, 0, 0, dpr, 0, 0);
    renderTask.value = page.render({ canvasContext: context, viewport });
    await renderTask.value.promise;
    renderTask.value = null;
  } catch (error) {
    if ((error as { name?: string }).name === "RenderingCancelledException") return;
    console.error("PDF.js failed to render PDF", error);
    errorMessage.value = error instanceof Error ? error.message : "PDF 渲染失败";
  }
}

// ---- All pages render (inline mode) ----
async function renderAllPagesInline() {
  if (!pdfDoc.value || !pagesContainerRef.value) return;

  cleanupAllPageTasks();
  clearPagesContainer();

  const numPages = pageCount.value;
  if (numPages === 0) return;

  const containerWidth = containerRef.value?.clientWidth ?? 320;
  if (containerWidth <= 0) return;

  const dpr = window.devicePixelRatio || 1;

  try {
    for (let i = 1; i <= numPages; i++) {
      const page = await pdfDoc.value.getPage(i);
      const baseViewport = page.getViewport({ scale: 1 });
      const scale = containerWidth / baseViewport.width;
      const viewport = page.getViewport({ scale });

      const canvas = document.createElement("canvas");
      canvas.width = Math.floor(viewport.width * dpr);
      canvas.height = Math.floor(viewport.height * dpr);
      canvas.style.display = "block";
      canvas.style.width = `${viewport.width}px`;
      canvas.style.height = `${viewport.height}px`;
      canvas.style.margin = i > 1 ? "4px 0 0 0" : "0";
      canvas.style.background = "#ffffff";

      const context = canvas.getContext("2d");
      if (!context) continue;

      context.setTransform(dpr, 0, 0, dpr, 0, 0);
      pagesContainerRef.value.appendChild(canvas);

      const task = page.render({ canvasContext: context, viewport });
      pageRenderTasks.push(task);
      await task.promise;
    }
  } catch (error) {
    if ((error as { name?: string }).name === "RenderingCancelledException") return;
    console.error("PDF.js inline render failed", error);
    errorMessage.value = error instanceof Error ? error.message : "PDF 渲染失败";
  }
}

// ---- Full mode controls ----
function previousPage() {
  currentPage.value = Math.max(1, currentPage.value - 1);
}
function nextPage() {
  currentPage.value = Math.min(pageCount.value, currentPage.value + 1);
}
function zoomOut() {
  scale.value = Math.max(0.5, Number((scale.value - 0.15).toFixed(2)));
}
function zoomIn() {
  scale.value = Math.min(2.5, Number((scale.value + 0.15).toFixed(2)));
}
</script>

<template>
  <div ref="containerRef" class="pdf-viewer" :class="{ inline: mode === 'inline' }">
    <!-- Toolbar: full mode only -->
    <div v-if="mode === 'full'" class="pdf-toolbar">
      <button type="button" :disabled="currentPage <= 1" @click.stop="previousPage">上一页</button>
      <button type="button" :disabled="currentPage >= pageCount" @click.stop="nextPage">下一页</button>
      <span>第 {{ currentPage }} / {{ pageCount || 1 }} 页</span>
      <button type="button" @click.stop="zoomOut">缩小</button>
      <button type="button" @click.stop="zoomIn">放大</button>
    </div>

    <!-- Loading -->
    <div v-if="loading" class="pdf-state">加载中</div>

    <!-- Error: simplified for inline -->
    <div v-else-if="errorMessage && mode === 'inline'" class="pdf-error-inline">
      <span>{{ errorMessage }}</span>
    </div>

    <!-- Error: detailed debug for full -->
    <div v-else-if="errorMessage && mode === 'full'" class="pdf-debug">
      <strong>PDF.js 加载失败</strong>
      <span>{{ errorMessage }}</span>
      <dl>
        <dt>fetch</dt>
        <dd>{{ fetchProbeStatus || "未执行" }}</dd>
        <dt>loadSource</dt>
        <dd>{{ loadSource || "未加载" }}</dd>
        <dt>pdfUrl</dt>
        <dd>{{ pdfUrl }}</dd>
        <dt>localFile</dt>
        <dd>{{ localFilePath || "未提供" }}</dd>
        <dt>exists / file / size</dt>
        <dd>
          {{
            diagnostic
              ? `${diagnostic.localFileExists} / ${diagnostic.isFile} / ${diagnostic.fileSizeBytes ?? "unknown"}`
              : "无诊断"
          }}
        </dd>
        <dt>under data / instance</dt>
        <dd>
          {{ diagnostic ? `${diagnostic.underDataDir} / ${diagnostic.underInstanceDir}` : "无诊断" }}
        </dd>
        <dt>scope</dt>
        <dd>{{ diagnostic?.configuredAssetScope || "未知" }}</dd>
        <dt>canonical</dt>
        <dd>{{ diagnostic?.canonicalFilePath || "未知" }}</dd>
        <dt>dataDir</dt>
        <dd>{{ diagnostic?.dataDir || "未知" }}</dd>
        <dt>exeDir</dt>
        <dd>{{ diagnostic?.exeDir || "未知" }}</dd>
        <dt>diagnostic</dt>
        <dd>{{ diagnosticError || diagnostic?.errorMessage || "无 Rust 侧错误" }}</dd>
      </dl>
    </div>

    <!-- Full mode: single canvas -->
    <div v-else-if="mode === 'full'" class="pdf-canvas-wrap">
      <canvas ref="canvasRef" />
    </div>

    <!-- Inline mode: DOM container for multi-page canvases -->
    <div v-else ref="pagesContainerRef" class="pdf-inline-pages"></div>
  </div>
</template>

<style scoped>
.pdf-viewer {
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  height: 100%;
  min-height: 0;
  background: #111111;
}

.pdf-viewer.inline {
  display: block;
  min-height: 0;
  background: transparent;
}

.pdf-viewer.inline canvas {
  display: block;
  max-width: none;
  background: #ffffff;
}

/* Toolbar (full mode) */
.pdf-toolbar {
  display: flex;
  align-items: center;
  gap: 6px;
  min-height: 32px;
  padding: 5px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  color: var(--muted);
  font-size: 11px;
}

.pdf-toolbar button {
  height: 22px;
  padding: 0 7px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 5px;
  background: #202020;
  color: #d4d4d8;
  cursor: pointer;
  font-size: 11px;
}

.pdf-toolbar button:disabled {
  cursor: not-allowed;
  opacity: 0.45;
}

/* Full mode canvas wrap */
.pdf-canvas-wrap {
  position: relative;
  min-height: 0;
  overflow: auto;
  padding: 10px;
}

/* State labels */
.pdf-state {
  position: absolute;
  top: 10px;
  left: 10px;
  z-index: 1;
  color: #ffffff;
  font-size: 12px;
  font-weight: 700;
}

.pdf-debug {
  position: absolute;
  top: 10px;
  left: 10px;
  z-index: 1;
  right: 10px;
  max-height: calc(100% - 20px);
  overflow: auto;
  padding: 8px;
  border: 1px solid rgba(248, 113, 113, 0.35);
  border-radius: 6px;
  background: rgba(12, 12, 12, 0.92);
  color: #fca5a5;
  font-size: 11px;
  line-height: 1.35;
}

.pdf-debug strong,
.pdf-debug span,
.pdf-debug dt,
.pdf-debug dd {
  display: block;
}

.pdf-debug dl {
  display: grid;
  grid-template-columns: 92px minmax(0, 1fr);
  gap: 3px 8px;
  margin: 8px 0 0;
}

.pdf-debug dt {
  color: #fbbf24;
}

.pdf-debug dd {
  min-width: 0;
  margin: 0;
  overflow-wrap: anywhere;
  color: #fecaca;
}

/* Inline mode */
.pdf-error-inline {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 80px;
  padding: 12px;
  color: #fca5a5;
  font-size: 12px;
  text-align: center;
}

.pdf-inline-pages {
  display: flex;
  flex-direction: column;
  min-height: 0;
}
</style>
