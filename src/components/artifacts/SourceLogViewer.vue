<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { readTextFile } from "../../services/fileService";
import { openPath } from "../../services/settingsService";

type TextFileTarget = {
  label: string;
  path?: string | null;
};

const props = defineProps<{
  title: string;
  targets: TextFileTarget[];
}>();

defineEmits<{
  close: [];
}>();

const availableTargets = computed(() => props.targets.filter((target) => target.path));
const activePath = ref(availableTargets.value[0]?.path ?? "");
const text = ref("");
const loading = ref(false);
const errorMessage = ref("");

watch(
  [activePath, availableTargets],
  async () => {
    if (!activePath.value && availableTargets.value[0]?.path) {
      activePath.value = availableTargets.value[0].path;
    }
    await loadText();
  },
  { immediate: true },
);

async function loadText() {
  if (!activePath.value) {
    text.value = "";
    return;
  }
  loading.value = true;
  errorMessage.value = "";
  try {
    text.value = await readTextFile(activePath.value);
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error);
  } finally {
    loading.value = false;
  }
}

async function copyText() {
  await navigator.clipboard.writeText(text.value);
}

async function openContainingDirectory() {
  if (!activePath.value) return;
  await openPath(parentPath(activePath.value));
}

function parentPath(path: string) {
  const index = Math.max(path.lastIndexOf("\\"), path.lastIndexOf("/"));
  return index > 0 ? path.slice(0, index) : path;
}
</script>

<template>
  <Teleport to="body">
    <div class="viewer-backdrop" @click.self="$emit('close')">
      <section class="viewer-panel">
        <header>
          <h1>{{ title }}</h1>
          <button type="button" @click="$emit('close')">×</button>
        </header>
        <nav>
          <button
            v-for="target in availableTargets"
            :key="target.path || target.label"
            type="button"
            :class="{ active: activePath === target.path }"
            @click="activePath = target.path || ''"
          >
            {{ target.label }}
          </button>
        </nav>
        <div class="path-row">
          <code>{{ activePath }}</code>
          <button type="button" :disabled="!text" @click="copyText">复制文本</button>
          <button type="button" :disabled="!activePath" @click="openContainingDirectory">打开目录</button>
        </div>
        <main>
          <pre v-if="loading">加载中</pre>
          <pre v-else-if="errorMessage" class="error">{{ errorMessage }}</pre>
          <pre v-else>{{ text }}</pre>
        </main>
      </section>
    </div>
  </Teleport>
</template>

<style scoped>
.viewer-backdrop {
  position: fixed;
  inset: 0;
  z-index: 50;
  display: grid;
  place-items: center;
  background: rgba(0, 0, 0, 0.62);
}

.viewer-panel {
  display: grid;
  grid-template-rows: auto auto auto minmax(0, 1fr);
  width: min(920px, calc(100vw - 36px));
  height: min(740px, calc(100vh - 36px));
  border: 1px solid var(--border);
  border-radius: 8px;
  background: #151515;
  color: var(--text);
}

header,
.path-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 10px 12px;
  border-bottom: 1px solid var(--border);
}

h1 {
  margin: 0;
  overflow: hidden;
  font-size: 14px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

nav {
  display: flex;
  gap: 6px;
  padding: 8px 12px;
  border-bottom: 1px solid var(--border);
}

button {
  height: 28px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: #202020;
  color: var(--text);
  cursor: pointer;
}

button.active {
  border-color: var(--accent);
}

button:disabled {
  cursor: not-allowed;
  opacity: 0.45;
}

.path-row code {
  min-width: 0;
  flex: 1;
  overflow: hidden;
  color: var(--muted);
  font-size: 11px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

main {
  min-height: 0;
  overflow: auto;
  padding: 12px;
}

pre {
  margin: 0;
  color: #d4d4d8;
  font-family: "JetBrains Mono", "Cascadia Mono", Consolas, monospace;
  font-size: 12px;
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-word;
}

.error {
  color: #f87171;
}
</style>
