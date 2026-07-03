export type SessionSourceKind = "mcp" | "manual" | "mock";

export type SessionClientName = "ZCode" | "Codex" | "Cursor" | "Unknown";

export type ArtifactKind = "image" | "pdf" | "latex" | "svg";

export type ArtifactStatus = "received" | "rendering" | "compiling" | "finished" | "failed";

export type ArtifactPreviewType = "large" | "small";

export type LayoutSize = "small" | "medium" | "wide";

export type SidebarMode = "groups" | "projects";

export type GalleryView = {
  sidebarMode: SidebarMode;
  groups: GalleryGroupView[];
  projects: GalleryProjectView[];
  sessions: GallerySessionView[];
  selectedSessionId?: string | null;
};

export type GalleryGroupView = {
  id: string;
  name: string;
  sessionIds: string[];
  sessionCount: number;
};

export type GalleryProjectView = {
  id: string;
  name: string;
  path: string;
  sessionIds: string[];
  sessionCount: number;
};

export type GallerySessionView = {
  id: string;
  title: string;
  sourceKind: SessionSourceKind;
  clientName: SessionClientName;
  groupName: string;
  projectName: string;
  projectPath: string;
  createdAt: string;
  updatedAt: string;
  updatedAtLabel: string;
  artifactCount: number;
  turns: GalleryTurnView[];
};

export type GalleryTurnView = {
  id: string;
  index: number;
  hint: string;
  createdAt: string;
  collapsed: boolean;
  artifacts: GalleryArtifactView[];
};

export type GalleryArtifactView = {
  id: string;
  title: string;
  kind: ArtifactKind;
  status: ArtifactStatus;
  previewType: ArtifactPreviewType;
  imageUrl?: string | null;
  localFilePath?: string | null;
  assetUrl?: string | null;
  pdfUrl?: string | null;
  pdfLocalFilePath?: string | null;
  pdfAssetUrl?: string | null;
  logFilePath?: string | null;
  stdoutPath?: string | null;
  stderrPath?: string | null;
  svg?: string | null;
  latexCode?: string | null;
  sourceFilePath?: string | null;
  mimeType?: string | null;
  latexEngine?: LatexEngine | null;
  compileElapsedMs?: number | null;
  errorMessage?: string | null;
  createdAt: string;
};

export type McpServerStatus = {
  running: boolean;
  status?: "running" | "stopped" | "failed";
  host?: string;
  url?: string | null;
  port?: number | null;
  errorMessage?: string | null;
};

export type SidecarConfig = {
  mcp: {
    host: string;
    port: number;
  };
  latex: {
    engine: LatexEngine;
    compileTimeoutSeconds: number;
  };
  gallery: {
    defaultSidebarMode: SidebarMode;
  };
};

export type SidecarConfigView = {
  config: SidecarConfig;
  instanceFolderName: string;
  instanceDir: string;
  configPath: string;
  dataDir: string;
  galleryStatePath: string;
  galleryEventsPath: string;
  artifactsDir: string;
  logsDir: string;
  debugArtifactsDir: string;
  lockPath: string;
};

export type LatexEngine = "pdflatex" | "xelatex" | "lualatex";

export type LatexDoctorReport = {
  checkedAt: string;
  defaultEngine: LatexEngine;
  tools: LatexToolStatus[];
  packages: LatexPackageStatus[];
  smokeTest?: LatexCompileResult | null;
};

export type LatexToolStatus = {
  name: string;
  status: "found" | "missing" | "failed";
  path?: string | null;
  version?: string | null;
  errorMessage?: string | null;
};

export type LatexPackageStatus = {
  name: string;
  status: "found" | "missing" | "failed";
  path?: string | null;
  errorMessage?: string | null;
};

export type LatexCompileResult = {
  ok: boolean;
  artifactId: string;
  engine: LatexEngine;
  elapsedMs: number;
  workDir: string;
  mainTexPath: string;
  sourceFilePath: string;
  pdfPath?: string | null;
  pdfFilePath?: string | null;
  logPath?: string | null;
  logFilePath?: string | null;
  stdoutPath: string;
  stderrPath: string;
  exitCode?: number | null;
  errorMessage?: string | null;
  finishedAt: string;
};

export type Session = GallerySessionView;
export type Turn = GalleryTurnView;
export type Artifact = GalleryArtifactView;
