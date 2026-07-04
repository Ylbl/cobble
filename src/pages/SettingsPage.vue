<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import {
  getSidecarConfig,
  openPath,
  restartMcpServer,
  stopMcpServer,
  runLatexEnvironmentCheck,
  runLatexSmokeTest,
  updateSidecarConfig,
} from "../services/settingsService";
import type { GalleryView, LatexDoctorReport, McpServerStatus, SidecarConfigView } from "../types/gallery";

const props = defineProps<{
  mcpStatus: McpServerStatus;
}>();

const emit = defineEmits<{
  close: [];
  "mcp-status": [status: McpServerStatus];
  "gallery-view": [view: GalleryView];
}>();

const activeTab = ref<"instance" | "mcp" | "latex" | "environment" | "data">("mcp");
const configView = ref<SidecarConfigView | null>(null);
const report = ref<LatexDoctorReport | null>(null);
const busy = ref(false);
const message = ref("");

const currentMcpUrl = computed(() => {
  if (!configView.value) return "";
  return `http://${configView.value.config.mcp.host}:${configView.value.config.mcp.port}/mcp`;
});

onMounted(async () => {
  configView.value = await getSidecarConfig();
});

async function saveSettings() {
  if (!configView.value) return;
  busy.value = true;
  try {
    configView.value = await updateSidecarConfig(configView.value);
    message.value = "配置已保存。MCP 端口变更后需要点击 Restart MCP Server。";
  } finally {
    busy.value = false;
  }
}

async function restartMcp() {
  busy.value = true;
  try {
    const status = await restartMcpServer();
    emit("mcp-status", status);
    message.value = status.status === "failed" ? status.errorMessage ?? "MCP Server 启动失败" : "MCP Server 已重启";
  } finally {
    busy.value = false;
  }
}

async function stopMcp() {
  busy.value = true;
  try {
    const status = await stopMcpServer();
    emit("mcp-status", status);
    message.value = "MCP Server 已停止";
  } finally {
    busy.value = false;
  }
}

async function copyMcpUrl() {
  await navigator.clipboard.writeText(currentMcpUrl.value);
  message.value = "MCP URL 已复制";
}

async function runCheck() {
  busy.value = true;
  try {
    report.value = await runLatexEnvironmentCheck();
  } finally {
    busy.value = false;
  }
}

async function runSmoke() {
  busy.value = true;
  try {
    const view = await runLatexSmokeTest();
    emit("gallery-view", view);
    message.value = "LaTeX smoke test 已完成，结果已写入 Gallery。";
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div class="settings-overlay">
    <section class="settings-panel">
      <header>
        <h1>设置</h1>
        <button type="button" @click="$emit('close')">×</button>
      </header>
      <div class="settings-body">
        <aside class="settings-nav">
          <button :class="{ active: activeTab === 'instance' }" @click="activeTab = 'instance'">实例</button>
          <button :class="{ active: activeTab === 'mcp' }" @click="activeTab = 'mcp'">MCP</button>
          <button :class="{ active: activeTab === 'latex' }" @click="activeTab = 'latex'">LaTeX</button>
          <button :class="{ active: activeTab === 'environment' }" @click="activeTab = 'environment'">环境</button>
          <button :class="{ active: activeTab === 'data' }" @click="activeTab = 'data'">数据目录</button>
        </aside>
        <main v-if="configView" class="settings-main">

          <!-- 实例 -->
          <section v-if="activeTab === 'instance'" class="settings-section">
            <div class="kv"><span>Instance Folder</span><code>{{ configView.instanceFolderName }}</code></div>
            <div class="kv"><span>Instance Directory</span><code>{{ configView.instanceDir }}</code></div>
            <div class="kv"><span>Config Path</span><code>{{ configView.configPath }}</code></div>
            <div class="kv"><span>Data Directory</span><code>{{ configView.dataDir }}</code></div>
            <div class="actions">
              <button type="button" @click="openPath(configView.instanceDir)">Open Instance Directory</button>
              <button type="button" @click="openPath(configView.configPath)">Open Config File</button>
              <button type="button" @click="openPath(configView.dataDir)">Open Data Directory</button>
            </div>
            <p>当前实例由 exe 所在文件夹决定。配置文件固定为 exe 同级 sidecar.config.json。数据目录固定为 exe 同级 data/。如需创建新实例，请复制整个 Sidecar 文件夹，并修改新文件夹中的 MCP 端口。</p>
          </section>

          <!-- MCP -->
          <section v-if="activeTab === 'mcp'" class="settings-section">
            <label>Host<input v-model="configView.config.mcp.host" /></label>
            <label>Port<input v-model.number="configView.config.mcp.port" type="number" min="1" max="65535" /></label>
            <label>
              MCP Instructions<span class="hint">（发送给 AI 的提示词，留空则用默认）</span>
              <textarea v-model="configView.config.mcp.instructions" rows="14" placeholder="留空使用默认提示词" />
            </label>
            <div class="kv"><span>Current MCP URL</span><code>{{ currentMcpUrl }}</code></div>
            <div class="kv"><span>MCP Status</span><strong :class="props.mcpStatus.status">{{ props.mcpStatus.status ?? (props.mcpStatus.running ? "running" : "stopped") }}</strong></div>
            <p v-if="props.mcpStatus.status === 'failed'" class="error">{{ props.mcpStatus.errorMessage }}</p>
            <div class="actions">
              <button type="button" @click="copyMcpUrl">Copy MCP URL</button>
              <button type="button" @click="restartMcp">Restart MCP Server</button>
              <button v-if="props.mcpStatus.running" type="button" @click="stopMcp">Stop MCP</button>
            </div>
          </section>

          <!-- LaTeX -->
          <section v-if="activeTab === 'latex'" class="settings-section">
            <label>默认 LaTeX 引擎
              <select v-model="configView.config.latex.engine">
                <option value="pdflatex">pdflatex</option>
                <option value="xelatex">xelatex</option>
                <option value="lualatex">lualatex</option>
              </select>
            </label>
            <label>Compile timeout seconds<input v-model.number="configView.config.latex.compileTimeoutSeconds" type="number" min="5" /></label>
          </section>

          <!-- 环境 -->
          <section v-if="activeTab === 'environment'" class="settings-section">
            <div class="actions">
              <button type="button" @click="runCheck">Run Environment Check</button>
              <button type="button" @click="runSmoke">Run LaTeX Smoke Test</button>
            </div>
            <div v-if="report" class="report">
              <div class="kv"><span>最近检测</span><code>{{ report.checkedAt }}</code></div>
              <h2>工具</h2>
              <div v-for="tool in report.tools" :key="tool.name" class="result-row"><strong :class="tool.status">{{ tool.name }} · {{ tool.status }}</strong><span>{{ tool.version || tool.errorMessage || tool.path }}</span></div>
              <h2>包</h2>
              <div v-for="pkg in report.packages" :key="pkg.name" class="result-row"><strong :class="pkg.status">{{ pkg.name }} · {{ pkg.status }}</strong><span>{{ pkg.path || pkg.errorMessage }}</span></div>
            </div>
            <p v-else>尚未检测</p>
          </section>

          <!-- 数据目录 -->
          <section v-if="activeTab === 'data'" class="settings-section">
            <div class="kv"><span>gallery-state.json</span><code>{{ configView.galleryStatePath }}</code></div>
            <div class="kv"><span>gallery-events.jsonl</span><code>{{ configView.galleryEventsPath }}</code></div>
            <div class="kv"><span>artifacts/</span><code>{{ configView.artifactsDir }}</code></div>
            <div class="kv"><span>logs/</span><code>{{ configView.logsDir }}</code></div>
            <div class="kv"><span>debug-artifacts/</span><code>{{ configView.debugArtifactsDir }}</code></div>
            <div class="kv"><span>.sidecar.lock</span><code>{{ configView.lockPath }}</code></div>
            <div class="kv"><span>mcp-sessions.json</span><code>{{ configView.mcpSessionsPath }}</code></div>
            <button type="button" @click="openPath(configView.logsDir)">Open Logs Directory</button>
            <button v-if="report?.smokeTest?.workDir" type="button" @click="openPath(report.smokeTest.workDir)">打开最近 smoke test 目录</button>
          </section>

        </main>
      </div>
      <footer>
        <span>{{ message }}</span>
        <button type="button" :disabled="busy || !configView" @click="saveSettings">保存设置</button>
      </footer>
    </section>
  </div>
</template>

<style scoped>
.settings-overlay {
  position: fixed;
  inset: 0;
  z-index: 40;
  display: grid;
  place-items: center;
  background: rgba(0, 0, 0, 0.55);
}

.settings-panel {
  display: flex;
  flex-direction: column;
  width: min(860px, calc(100vw - 80px));
  height: min(640px, calc(100vh - 120px));
  border: 1px solid var(--border);
  border-radius: 8px;
  background: #121212;
  color: var(--text);
}

header, footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
  flex: 0 0 auto;
}
header { border-bottom: 1px solid var(--border); }
footer { border-top: 1px solid var(--border); }

h1, h2 { margin: 0; font-size: 14px; }

.settings-body {
  display: flex;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.settings-nav {
  width: 130px;
  flex: 0 0 130px;
  border-right: 1px solid rgba(255, 255, 255, 0.08);
  padding: 8px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  overflow-y: auto;
}

.settings-nav::-webkit-scrollbar {
  width: 4px;
}
.settings-nav::-webkit-scrollbar-track {
  background: transparent;
}
.settings-nav::-webkit-scrollbar-thumb {
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.10);
}

.settings-nav button {
  width: 100%;
  text-align: left;
  padding: 8px 10px;
  border-radius: 6px;
  border: 1px solid transparent;
  background: transparent;
  color: #a7a7a7;
  cursor: pointer;
  font-size: 12px;
}

.settings-nav button.active {
  color: #fff;
  background: rgba(245, 158, 11, 0.12);
  border-color: rgba(245, 158, 11, 0.55);
}

.settings-nav button:hover:not(.active) {
  background: rgba(255, 255, 255, 0.04);
  color: #d4d4d8;
}

.settings-main {
  flex: 1;
  min-width: 0;
  padding: 14px;
  overflow-y: auto;
}

.settings-main::-webkit-scrollbar {
  width: 6px;
}

.settings-main::-webkit-scrollbar-track {
  background: transparent;
}

.settings-main::-webkit-scrollbar-thumb {
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.13);
}

.settings-section {
  display: grid;
  gap: 12px;
}

label {
  display: grid;
  gap: 5px;
  color: var(--subtle);
  font-size: 12px;
}

.hint {
  color: var(--subtle);
  font-size: 10px;
  font-weight: 400;
}

/* Dark inputs */
input, textarea, select {
  height: 32px;
  padding: 0 9px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 6px;
  background: #1f1f1f;
  color: #e8e8e8;
  font-size: 13px;
}

textarea {
  height: auto;
  min-height: 160px;
  padding: 8px 9px;
  resize: vertical;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", monospace;
  font-size: 11px;
  line-height: 1.45;
}

textarea::-webkit-scrollbar {
  width: 8px;
}
textarea::-webkit-scrollbar-track {
  background: #1a1a1a;
}
textarea::-webkit-scrollbar-thumb {
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.13);
}

input:focus, textarea:focus, select:focus {
  outline: none;
  border-color: rgba(245, 158, 11, 0.75);
  box-shadow: 0 0 0 1px rgba(245, 158, 11, 0.25);
}

button {
  height: 30px;
  padding: 0 10px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: #202020;
  color: var(--text);
  cursor: pointer;
  font-size: 12px;
}

button:hover { background: #2a2a2a; }

.kv, .result-row {
  display: grid;
  gap: 4px;
  padding: 8px 10px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 6px;
  background: #1a1a1a;
}

code, .result-row span {
  color: var(--muted);
  font-size: 12px;
  word-break: break-all;
}

.actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

p {
  margin: 0;
  color: var(--muted);
  font-size: 11px;
  line-height: 1.45;
}

.error, .failed, .missing { color: #f87171; }
.running, .found { color: #86efac; }
</style>

<style>
.settings-panel textarea::-webkit-scrollbar { width: 8px; }
.settings-panel textarea::-webkit-scrollbar-track { background: #1a1a1a; }
.settings-panel textarea::-webkit-scrollbar-thumb { border-radius: 999px; background: rgba(255, 255, 255, 0.13); }
</style>
