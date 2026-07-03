<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref, shallowRef, watch } from "vue";
import * as pdfjsLib from "pdfjs-dist";
import pdfWorkerUrl from "pdfjs-dist/build/pdf.worker.mjs?url";
import { getCachedPdfDocument, getCachedPdfDocumentFromBytes } from "../../services/pdfDocumentCache";
import { pdfRenderQueue } from "../../services/pdfRenderQueue";

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
    renderAllPages: false,
  },
);

pdfjsLib.GlobalWorkerOptions.workerSrc = pdfWorkerUrl;

const containerRef = ref<HTMLDivElement | null>(null);
const canvasRef = ref<HTMLCanvasElement | null>(null);
const pagesContainerRef = ref<HTMLDivElement | null>(null);

const pdfDoc = shallowRef<pdfjsLib.PDFDocumentProxy | null>(null);
const renderTask = shallowRef<any>(null);
const pageRenderTasks: any[] = [];
const currentPage = ref(1);
const pageCount = ref(0);
const scale = ref(props.initialScale);
const loading = ref(false);
const errorMessage = ref("");
const diagnosticError = ref("");

let lastRenderedWidth = 0;
let lastRenderedDpr = 0;
let lastPdfKey = "";
let ro: ResizeObserver | null = null;
let resizeTimer: ReturnType<typeof setTimeout> | null = null;
let hiResTimer: ReturnType<typeof setTimeout> | null = null;
let isLoadingPdf = false;

const isInline = () => props.mode === "inline";
const pdfKey = () => props.pdfUrl || props.localFilePath || "";

function cleanupRenderTask() {
  if (renderTask.value) { renderTask.value.cancel(); renderTask.value = null; }
}
function cleanupAllPageTasks() {
  for (const task of pageRenderTasks) { if (task) task.cancel(); }
  pageRenderTasks.length = 0;
}
function clearPagesContainer() {
  if (pagesContainerRef.value) pagesContainerRef.value.innerHTML = "";
}
function resetPdfState() {
  cleanupRenderTask(); cleanupAllPageTasks(); clearPagesContainer();
  if (hiResTimer) clearTimeout(hiResTimer);
  pdfDoc.value = null; currentPage.value = 1; pageCount.value = 0;
  errorMessage.value = ""; diagnosticError.value = "";
  lastRenderedWidth = 0; lastRenderedDpr = 0; lastPdfKey = "";
}

// Wait for container to have non-zero width, then load
function scheduleRenderWhenReady() {
  if (!props.pdfUrl && !props.localFilePath) return;
  nextTick(() => {
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        const width = containerRef.value?.clientWidth ?? 0;
        if (width <= 0) { setTimeout(scheduleRenderWhenReady, 100); return; }
        if (isInline()) { if (!isLoadingPdf) loadPdfInline(); }
        else { loadPdfFull(); }
      });
    });
  });
}

watch(() => props.pdfUrl, () => scheduleRenderWhenReady(), { immediate: true });
watch([currentPage, scale], async () => { if (!isInline()) await renderPage(); });

function setupResizeObserver() {
  if (!containerRef.value) return;
  ro = new ResizeObserver(() => {
    if (resizeTimer) clearTimeout(resizeTimer);
    resizeTimer = setTimeout(() => {
      const w = containerRef.value?.clientWidth ?? 0;
      if (w <= 0) return;
      if (isInline() && pdfDoc.value && pageCount.value > 0 && Math.abs(w - lastRenderedWidth) >= 8) {
        renderInlinePage();
      } else if (!isInline() && pdfDoc.value && pageCount.value > 0) {
        renderPage();
      }
    }, 120);
  });
  ro.observe(containerRef.value);
}

onMounted(() => setupResizeObserver());
onBeforeUnmount(() => {
  ro?.disconnect(); ro = null;
  if (resizeTimer) clearTimeout(resizeTimer);
  if (hiResTimer) clearTimeout(hiResTimer);
  resetPdfState();
});

async function loadPdfInline() {
  if (isLoadingPdf) return;
  isLoadingPdf = true; resetPdfState(); loading.value = true;
  try {
    if (props.localFilePath && props.artifactId) {
      try {
        const { readBinaryFile } = await import("../../services/fileService");
        const pdfBytes = await readBinaryFile(props.localFilePath);
        pdfDoc.value = await getCachedPdfDocumentFromBytes(`local:${props.localFilePath}`, pdfBytes);
      } catch { pdfDoc.value = null; }
    }
    if (!pdfDoc.value && props.pdfUrl) pdfDoc.value = await getCachedPdfDocument(props.pdfUrl);
    if (!pdfDoc.value) { errorMessage.value = "PDF load failed"; return; }
    pageCount.value = pdfDoc.value.numPages;
    await nextTick();
    await renderInlinePage();
  } catch (error) {
    console.error("PDF.js failed", error);
    errorMessage.value = error instanceof Error ? error.message : "PDF error";
  } finally { loading.value = false; isLoadingPdf = false; }
}

async function loadPdfFull() {
  resetPdfState(); loading.value = true;
  try {
    if (props.localFilePath && props.artifactId) {
      try {
        const { readBinaryFile } = await import("../../services/fileService");
        pdfDoc.value = await getCachedPdfDocumentFromBytes(`local:${props.localFilePath}`, await readBinaryFile(props.localFilePath));
      } catch { pdfDoc.value = null; }
    }
    if (!pdfDoc.value && props.pdfUrl) pdfDoc.value = await getCachedPdfDocument(props.pdfUrl);
    if (!pdfDoc.value) { errorMessage.value = "PDF load failed"; return; }
    pageCount.value = pdfDoc.value.numPages;
    await nextTick(); await renderPage();
  } catch (error) {
    console.error("PDF.js failed", error);
    errorMessage.value = error instanceof Error ? error.message : "PDF error";
  } finally { loading.value = false; }
}

function getDpr(stage: "preview" | "final"): number {
  const d = window.devicePixelRatio || 1;
  if (stage === "preview") return 1;
  return isInline() ? Math.min(d, 1.5) : Math.min(d, 2);
}
function needsRerender(w: number, dpr: number, key: string) {
  return key !== lastPdfKey || Math.abs(w - lastRenderedWidth) >= 8 || dpr !== lastRenderedDpr;
}
function markRendered(w: number, dpr: number, key: string) {
  lastRenderedWidth = w; lastRenderedDpr = dpr; lastPdfKey = key;
}

async function renderInlinePage() {
  if (!pdfDoc.value || !pagesContainerRef.value) return;
  const w = containerRef.value?.clientWidth ?? 320;
  if (w <= 0) return;
  const key = pdfKey(); const previewDpr = getDpr("preview");
  if (!needsRerender(w, previewDpr, key)) return;
  cleanupAllPageTasks(); clearPagesContainer();
  try {
    const page = await pdfDoc.value.getPage(1);
    const vp = page.getViewport({ scale: w / page.getViewport({ scale: 1 }).width });
    const c = document.createElement("canvas");
    c.width = Math.floor(vp.width * previewDpr); c.height = Math.floor(vp.height * previewDpr);
    c.style.cssText = `display:block;width:${vp.width}px;height:${vp.height}px;background:#fff`;
    const ctx = c.getContext("2d"); if (!ctx) return;
    ctx.setTransform(previewDpr, 0, 0, previewDpr, 0, 0);
    pagesContainerRef.value.appendChild(c);
    markRendered(w, previewDpr, key);
    pdfRenderQueue.enqueue(async () => {
      const t = page.render({ canvasContext: ctx, viewport: vp, canvas: c });
      pageRenderTasks.push(t); await t.promise;
    });
    const fdpr = getDpr("final");
    if (fdpr > previewDpr) {
      if (hiResTimer) clearTimeout(hiResTimer);
      hiResTimer = setTimeout(() => renderInlineHiRes(page, w, fdpr, key), 300);
    }
  } catch (error) {
    if ((error as any)?.name === "RenderingCancelledException") return;
    errorMessage.value = error instanceof Error ? error.message : "PDF render error";
  }
}

async function renderInlineHiRes(page: any, w: number, dpr: number, key: string) {
  if (!pagesContainerRef.value || !needsRerender(w, dpr, key)) return;
  cleanupAllPageTasks(); clearPagesContainer();
  const vp = page.getViewport({ scale: w / page.getViewport({ scale: 1 }).width });
  const c = document.createElement("canvas");
  c.width = Math.floor(vp.width * dpr); c.height = Math.floor(vp.height * dpr);
  c.style.cssText = `display:block;width:${vp.width}px;height:${vp.height}px;background:#fff`;
  const ctx = c.getContext("2d"); if (!ctx) return;
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  pagesContainerRef.value.appendChild(c);
  markRendered(w, dpr, key);
  pdfRenderQueue.enqueue(async () => {
    const t = page.render({ canvasContext: ctx, viewport: vp, canvas: c });
    pageRenderTasks.push(t); await t.promise;
  });
}

async function renderPage() {
  if (!pdfDoc.value || !canvasRef.value) return;
  cleanupRenderTask();
  try {
    const page = await pdfDoc.value.getPage(currentPage.value);
    const vp = page.getViewport({ scale: scale.value });
    const ctx = canvasRef.value!.getContext("2d"); if (!ctx) return;
    const dpr = getDpr("final");
    canvasRef.value!.width = Math.floor(vp.width * dpr);
    canvasRef.value!.height = Math.floor(vp.height * dpr);
    canvasRef.value!.style.width = `${vp.width}px`;
    canvasRef.value!.style.height = `${vp.height}px`;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    renderTask.value = page.render({ canvasContext: ctx, viewport: vp, canvas: canvasRef.value! });
    await renderTask.value.promise; renderTask.value = null;
  } catch (error) {
    if ((error as any)?.name === "RenderingCancelledException") return;
    errorMessage.value = error instanceof Error ? error.message : "PDF render error";
  }
}

function previousPage() { currentPage.value = Math.max(1, currentPage.value - 1); }
function nextPage() { currentPage.value = Math.min(pageCount.value, currentPage.value + 1); }
function zoomOut() { scale.value = Math.max(0.5, Number((scale.value - 0.15).toFixed(2))); }
function zoomIn() { scale.value = Math.min(2.5, Number((scale.value + 0.15).toFixed(2))); }
</script>

<template>
  <div ref="containerRef" class="pdf-viewer" :class="{ inline: mode === 'inline' }">
    <div v-if="mode === 'full'" class="pdf-toolbar">
      <button type="button" :disabled="currentPage <= 1" @click.stop="previousPage">上一页</button>
      <button type="button" :disabled="currentPage >= pageCount" @click.stop="nextPage">下一页</button>
      <span>第 {{ currentPage }} / {{ pageCount || 1 }} 页</span>
      <button type="button" @click.stop="zoomOut">缩小</button>
      <button type="button" @click.stop="zoomIn">放大</button>
    </div>
    <div v-show="loading" class="pdf-state">加载中</div>
    <div v-if="errorMessage && mode === 'inline'" class="pdf-error-inline"><span>{{ errorMessage }}</span></div>
    <div v-else-if="errorMessage && mode === 'full'" class="pdf-debug">
      <strong>PDF.js 加载失败</strong><span>{{ errorMessage }}</span>
      <dl><dt>pdfUrl</dt><dd>{{ pdfUrl }}</dd><dt>localFile</dt><dd>{{ localFilePath || "未提供" }}</dd><dt>diagnostic</dt><dd>{{ diagnosticError || "无" }}</dd></dl>
    </div>
    <div v-else-if="mode === 'full'" class="pdf-canvas-wrap"><canvas ref="canvasRef" /></div>
    <div v-else ref="pagesContainerRef" class="pdf-inline-pages"></div>
  </div>
</template>

<style scoped>
.pdf-viewer { display: grid; grid-template-rows: auto minmax(0, 1fr); height: 100%; min-height: 0; background: #111111; }
.pdf-viewer.inline { display: block; min-height: 0; background: transparent; }
.pdf-toolbar { display: flex; align-items: center; gap: 6px; min-height: 32px; padding: 5px; border-bottom: 1px solid rgba(255,255,255,0.08); color: var(--muted); font-size: 11px; }
.pdf-toolbar button { height: 22px; padding: 0 7px; border: 1px solid rgba(255,255,255,0.12); border-radius: 5px; background: #202020; color: #d4d4d8; cursor: pointer; font-size: 11px; }
.pdf-toolbar button:disabled { cursor: not-allowed; opacity: 0.45; }
.pdf-canvas-wrap { position: relative; min-height: 0; overflow: auto; padding: 10px; }
.pdf-state { position: absolute; top: 10px; left: 10px; z-index: 1; color: #fff; font-size: 12px; font-weight: 700; }
.pdf-debug { position: absolute; top: 10px; left: 10px; z-index: 1; right: 10px; max-height: calc(100% - 20px); overflow: auto; padding: 8px; border: 1px solid rgba(248,113,113,0.35); border-radius: 6px; background: rgba(12,12,12,0.92); color: #fca5a5; font-size: 11px; line-height: 1.35; }
.pdf-debug strong, .pdf-debug span, .pdf-debug dt, .pdf-debug dd { display: block; }
.pdf-debug dl { display: grid; grid-template-columns: 72px minmax(0, 1fr); gap: 3px 8px; margin: 8px 0 0; }
.pdf-debug dt { color: #fbbf24; }
.pdf-debug dd { min-width: 0; margin: 0; overflow-wrap: anywhere; color: #fecaca; }
.pdf-error-inline { display: flex; align-items: center; justify-content: center; min-height: 80px; padding: 12px; color: #fca5a5; font-size: 12px; text-align: center; }
.pdf-inline-pages { display: flex; flex-direction: column; min-height: 0; }
</style>
