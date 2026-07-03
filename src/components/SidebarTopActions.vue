<template>
  <div class="sidebar-top-actions">
    <div class="nav-strip">
      <span class="app-mark">Z</span>
      <button type="button" title="后退">‹</button>
      <button type="button" title="前进">›</button>
    </div>
    <button class="action-button" type="button">
      <span>⊕</span>
      <strong>新建任务</strong>
      <kbd>Ctrl+N</kbd>
    </button>
    <button class="action-button" type="button">
      <span>⌕</span>
      <strong>搜索</strong>
      <kbd>Ctrl+K</kbd>
    </button>
    <button class="action-button" type="button">
      <span>⌘</span>
      <strong>技能 / MCP</strong>
      <kbd>Ctrl+J</kbd>
    </button>
  </div>
</template>

<style scoped>
.sidebar-top-actions {
  padding: 10px 10px 8px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
}

.nav-strip {
  display: flex;
  align-items: center;
  gap: 6px;
  height: 24px;
  margin-bottom: 8px;
}

.app-mark {
  display: grid;
  width: 18px;
  height: 18px;
  place-items: center;
  border: 1px solid rgba(255, 255, 255, 0.15);
  border-radius: 4px;
  background: #101010;
  color: #ffffff;
  font-size: 11px;
  font-weight: 800;
}

.nav-strip button {
  display: grid;
  width: 20px;
  height: 20px;
  place-items: center;
  border: 0;
  border-radius: 5px;
  background: transparent;
  color: var(--subtle);
  cursor: pointer;
}

.nav-strip button:hover {
  background: rgba(255, 255, 255, 0.06);
  color: var(--text);
}

.action-button {
  display: grid;
  grid-template-columns: 18px minmax(0, 1fr) auto;
  align-items: center;
  width: 100%;
  height: 28px;
  margin-top: 6px;
  padding: 0 8px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 5px;
  background: #202020;
  color: var(--muted);
  cursor: pointer;
  text-align: left;
}

.action-button:hover {
  border-color: rgba(245, 158, 11, 0.55);
  background: #252525;
  color: var(--text);
}

.action-button strong {
  overflow: hidden;
  color: #e4e4e7;
  font-size: 12px;
  font-weight: 620;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.action-button kbd {
  color: #6f6f76;
  font-size: 10px;
  font-family: inherit;
}
</style>
