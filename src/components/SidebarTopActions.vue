<template>
  <div class="sidebar-top-actions" data-tauri-drag-region>
    <div class="brand-row">
      <span class="app-mark">⊞</span>
    </div>
  </div>
</template>

<style scoped>
.sidebar-top-actions {
  padding: 8px 10px 6px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  min-height: 30px;
  cursor: default;
}

.brand-row {
  display: flex;
  align-items: center;
  height: 22px;
}

.app-mark {
  display: grid;
  width: 22px;
  height: 22px;
  place-items: center;
  border: 1px solid rgba(255, 255, 255, 0.15);
  border-radius: 5px;
  background: #101010;
  color: #f59e0b;
  font-size: 13px;
  font-weight: 800;
}
</style>
