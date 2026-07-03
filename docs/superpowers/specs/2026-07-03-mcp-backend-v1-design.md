# MCP Backend v1 Design

> 2026-07-03 | Sidecar App 第一版后端与 MCP 接入

## 概述

实现最小闭环：外部 AI/MCP Client → 内置 Streamable HTTP MCP Server → display_artifact_turn tool → Rust 后端创建 session/turn/artifact → Tauri event 通知前端 → 前端实时展示。

**第一版范围**：仅支持 image artifact。不实现 LaTeX 编译、PDF.js、MiKTeX、标题提取、stdio MCP bridge。

---

## 一、三层类型体系

### MCP 协议层（干净，面向外部 AI）

```rust
// 输入
DisplayArtifactTurnInput {
    sidecar_session_id: Option<String>,
    session_title: Option<String>,
    turn_hint: Option<String>,
    artifacts: Vec<ArtifactInput>,
}
ArtifactInput {
    title: String,
    kind: ArtifactInputKind, // "image" | "latex" | "pdf" | "svg"
    image_url: Option<String>,
    latex_code: Option<String>,
    pdf_url: Option<String>,
    svg: Option<String>,
}

// 输出
DisplayArtifactTurnResult {
    ok: bool,
    sidecar_session_id: String,
    sidecar_turn_id: String,
    artifact_ids: Vec<String>,
    created_new_session: bool,
    displayed: bool,
    message: String,
    reuse_instruction: String,
}
```

### Rust Domain 层（干净领域模型）

```rust
ArtifactSession {
    id, title, source_kind: "mcp"|"manual"|"mock",
    client_name: Option<String>, // "ZCode"|"Codex"|"Cursor"|"Unknown"
    created_at, updated_at, turns
}
ArtifactTurn { id, index, hint, created_at, artifacts, collapsed }
ArtifactItem {
    id, title, kind: "image"|"pdf"|"latex"|"svg",
    status: "received"|"rendering"|"finished"|"failed",
    image_url, source_text, mime_type, file_extension, created_at
}
```

### 前端 UI 层（superset，保留现有字段 + 新增）

```ts
// 原字段保留
projectName, projectPath, previewType

// source 拆分为两个概念
sourceKind: "mcp" | "manual" | "mock"
clientName: "ZCode" | "Codex" | "Cursor" | "Unknown"

// kind 统一
ArtifactKind: "image" | "pdf" | "latex" | "svg"  // 不再用 "png"

// 新增
imageUrl, sourceText, mimeType, fileExtension, createdAt
```

### Adapter 层（`src/adapters/galleryAdapter.ts`）

后端 → 前端转换规则：
- `previewType`: artifactIndex === 0 ? "large" : "small"
- `projectName`: backend.projectName ?? ""
- `projectPath`: backend.projectPath ?? ""
- `clientName`: backend.clientName ?? "Unknown"
- `kind`: backend.kind（已是 "image"，无需转换）

---

## 二、Rust 文件结构

```
src-tauri/src/
 ├─ lib.rs              # Tauri app 入口，注册 commands/events/state
 ├─ main.rs             # Windows 子系统配置
 ├─ logging.rs          # tracing 初始化（终端 + 文件 JSON 日志）
 ├─ mcp/
 │   ├─ mod.rs
 │   ├─ http_server.rs  # Streamable HTTP MCP Server 启动
 │   ├─ tools.rs        # display_artifact_turn tool 定义
 │   └─ types.rs        # MCP 输入输出类型
 ├─ gallery/
 │   ├─ mod.rs
 │   ├─ state.rs        # GalleryState（RwLock<Vec<ArtifactSession>>）
 │   ├─ types.rs        # ArtifactSession / ArtifactTurn / ArtifactItem
 │   ├─ events.rs       # Tauri event emit
 │   └─ debug_artifacts.rs  # 本地 debug 产物保存
 └─ commands/
     ├─ mod.rs
     └─ gallery.rs      # list_gallery_sessions, get_mcp_server_status
```

---

## 三、关键依赖

```toml
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["fmt", "env-filter", "json"] }
tracing-appender = "0.2"
tokio = { version = "1", features = ["full"] }
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1"
thiserror = "2"
rmcp = { version = "2", features = ["server", "transport-streamable-http", "schemars"] }
schemars = "1"
```

---

## 四、数据流

```
MCP Client 调用 display_artifact_turn
    ↓
rmcp HTTP Server 接收请求
    ↓
tools.rs 解析参数，写日志
    ↓
gallery/state.rs 处理：
  - sidecarSessionId 存在 → 复用 session
  - 否则 → 创建新 session（title = sessionTitle 或 "未命名会话"）
  - 创建新 ArtifactTurn（index = session.turns.len() + 1）
  - 遍历 artifacts，只处理 kind=image
  - 创建 ArtifactItem（status=finished, imageUrl=input.imageUrl）
  - 更新 session.updatedAt
    ↓
gallery/debug_artifacts.rs 保存 run 目录
    ↓
gallery/events.rs emit:
  - gallery-updated（完整 sessions 列表）
  - artifact-created（sessionId, turnId, artifactIds）
    ↓
前端 listen → adapter 转换 → 更新 Vue 响应式状态
    ↓
UI 自动刷新
```

---

## 五、MCP Server

- 传输：Streamable HTTP
- 默认端口：39333（占用则尝试 39334/39335/39336）
- 写入状态文件：`src-tauri/debug-mcp/mcp-server.json`
- 前端通过 `get_mcp_server_status` command 读取状态并展示

---

## 六、日志系统

- **控制台**：pretty/compact 格式，人能看懂
- **文件日志**：JSON 格式，按天 rolling → `src-tauri/debug-logs/app.log.YYYY-MM-DD`
- **环境变量**：`SIDECAR_CONSOLE_LOG` / `SIDECAR_FILE_LOG`
- **WorkerGuard** 保存在 Tauri app state 中防止提前 drop
- **记录阶段**：启动 → 日志初始化 → MCP 启动 → tool 调用 → session/turn/artifact 创建 → debug 写入 → emit → 错误

---

## 七、Debug Artifacts

目录：`src-tauri/debug-artifacts/run-YYYYMMDD-HHMMSS-<short-id>/`

每次 MCP tool call 保存：
- `mcp-request.json`
- `resolved-session.json`
- `created-turn.json`
- `created-artifacts.json`
- `gallery-state-after.json`
- `tool-result.json`
- `summary.md`

---

## 八、Tauri 集成

### Commands
- `list_gallery_sessions` → `Vec<ArtifactSession>`
- `get_mcp_server_status` → `McpServerStatus { running, url, port }`

### Events (emit to frontend)
- `gallery-updated` → `{ sessions: [...] }`
- `artifact-created` → `{ sidecarSessionId, sidecarTurnId, artifactIds }`

### App State
```rust
tauri::Builder::default()
    .manage(GalleryState::new())
    .manage(McpServerState::new())
    .manage(LogGuard) // WorkerGuard
```

---

## 九、前端改动

### 新增文件
- `src/services/galleryService.ts` — invoke list_gallery_sessions
- `src/services/mcpStatusService.ts` — invoke get_mcp_server_status
- `src/adapters/galleryAdapter.ts` — 后端数据 → 前端 UI 数据

### 修改文件
- `src/types/gallery.ts` — 类型升级（kind: "image", sourceKind + clientName, 新增字段）
- `src/pages/GalleryPage.vue` — 替换 mock 数据为后端数据，监听 Tauri events
- 所有组件中 `kind === "png"` → `kind === "image"`
- `src/data/mockSessions.ts` — 更新 mock 数据适配新类型（开发阶段仍可用作 fallback）

### 保留不变
- 所有组件的 template 结构和 CSS 不变
- 左侧 session 列表 + 右侧按 turn 分组的布局不变
- TurnBlock 可折叠、ArtifactCard 展示等交互不变

---

## 十、第一版不实现

- LaTeX 编译（MiKTeX + latexmk + xelatex）
- PDF.js 渲染
- 标题自动提取
- stdio MCP bridge
- Codex / ZCode session adapter
- 安全限制
