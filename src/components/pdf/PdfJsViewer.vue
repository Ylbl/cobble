<script setup lang="ts">
import { nextTick, onBeforeUnmount, ref, shallowRef, watch } from "vue";
import * as pdfjsLib from "pdfjs-dist";
import pdfWorkerUrl from "pdfjs-dist/build/pdf.worker.mjs?url";
import type { PdfAssetDiagnostic } from "../../services/fileService";

const props = withDefaults(
  defineProps<{
    pdfUrl: string;
    artifactId?: string;
    localFilePath?: string | null;
    initialScale?: number;
  }>(),
  {
    initialScale: 1,
  },
);

pdfjsLib.GlobalWorkerOptions.workerSrc = pdfWorkerUrl;

const canvasRef = ref<HTMLCanvasElement | null>(null);
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

watch(
  () => props.pdfUrl,
  async () => {
    await loadPdf();
  },
  { immediate: true },
);

watch([currentPage, scale], async () => {
  await renderPage();
});

onBeforeUnmount(() => {
  cleanupRenderTask();
  pdfDoc.value?.destroy?.();
});

async function loadPdf() {
  cleanupRenderTask();
  pdfDoc.value?.destroy?.();
  pdfDoc.value = null;
  currentPage.value = 1;
  pageCount.value = 0;
  errorMessage.value = "";
  diagnostic.value = null;
  diagnosticError.value = "";
  fetchProbeStatus.value = "";
  loadSource.value = "";

  if (!props.pdfUrl) return;

  loading.value = true;
  try {
    if (props.artifactId) {
      const { diagnosePdfAssetPath, readBinaryFile, recordPdfjsPreviewRequested } = await import("../../services/fileService");
      try {
        await recordPdfjsPreviewRequested(props.artifactId, props.pdfUrl);
      } catch (error) {
        diagnosticError.value = `record failed: ${formatError(error)}`;
      }
      if (props.localFilePath) {
        try {
          diagnostic.value = await diagnosePdfAssetPath(props.artifactId, props.localFilePath, props.pdfUrl);
        } catch (error) {
          diagnosticError.value = `diagnose failed: ${formatError(error)}`;
        }
      }
      if (props.localFilePath) {
        try {
          const pdfBytes = await readBinaryFile(props.localFilePath);
          pdfDoc.value = await pdfjsLib.getDocument({ data: pdfBytes }).promise;
          loadSource.value = "local-bytes";
          pageCount.value = pdfDoc.value.numPages;
          await nextTick();
          await renderPage();
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
    await renderPage();
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

async function renderPage() {
  if (!pdfDoc.value || !canvasRef.value) return;

  cleanupRenderTask();
  try {
    const page = await pdfDoc.value.getPage(currentPage.value);
    const viewport = page.getViewport({ scale: scale.value });
    const canvas = canvasRef.value;
    const context = canvas.getContext("2d");
    if (!context) return;

    canvas.width = Math.floor(viewport.width);
    canvas.height = Math.floor(viewport.height);
    renderTask.value = page.render({ canvasContext: context, viewport });
    await renderTask.value.promise;
    renderTask.value = null;
  } catch (error) {
    if ((error as { name?: string }).name === "RenderingCancelledException") return;
    console.error("PDF.js failed to render PDF", error);
    errorMessage.value = error instanceof Error ? error.message : "PDF 渲染失败";
  }
}

function cleanupRenderTask() {
  if (renderTask.value) {
    renderTask.value.cancel();
    renderTask.value = null;
  }
}

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
  <div class="pdf-viewer">
    <div class="pdf-toolbar">
      <button type="button" :disabled="currentPage <= 1" @click.stop="previousPage">上一页</button>
      <button type="button" :disabled="currentPage >= pageCount" @click.stop="nextPage">下一页</button>
      <span>第 {{ currentPage }} / {{ pageCount || 1 }} 页</span>
      <button type="button" @click.stop="zoomOut">缩小</button>
      <button type="button" @click.stop="zoomIn">放大</button>
    </div>
    <div class="pdf-canvas-wrap">
      <span v-if="loading" class="pdf-state">加载中</span>
      <div v-else-if="errorMessage" class="pdf-debug">
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
            {{ diagnostic ? `${diagnostic.localFileExists} / ${diagnostic.isFile} / ${diagnostic.fileSizeBytes ?? "unknown"}` : "无诊断" }}
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
      <canvas ref="canvasRef" />
    </div>
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

.pdf-canvas-wrap {
  position: relative;
  min-height: 0;
  overflow: auto;
  padding: 10px;
}

canvas {
  display: block;
  max-width: none;
  margin: 0 auto;
  background: #ffffff;
}

.pdf-state,
.pdf-debug {
  position: absolute;
  top: 10px;
  left: 10px;
  z-index: 1;
  color: #ffffff;
  font-size: 12px;
  font-weight: 700;
}

.pdf-state.error {
  color: #f87171;
}

.pdf-debug {
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
</style>
