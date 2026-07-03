export type SessionSourceKind = "mcp" | "manual" | "mock";

export type SessionClientName = "ZCode" | "Codex" | "Cursor" | "Unknown";

export type ArtifactKind = "image" | "pdf" | "latex" | "svg";

export type ArtifactStatus = "received" | "rendering" | "finished" | "failed";

export type ArtifactPreviewType = "large" | "small";

export type GalleryView = {
  sessions: GallerySessionView[];
  selectedSessionId?: string | null;
};

export type GallerySessionView = {
  id: string;
  title: string;
  sourceKind: SessionSourceKind;
  clientName: SessionClientName;
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
  pdfUrl?: string | null;
  svg?: string | null;
  latexCode?: string | null;
  mimeType?: string | null;
  createdAt: string;
};

export type McpServerStatus = {
  running: boolean;
  url?: string | null;
  port?: number | null;
};

export type Session = GallerySessionView;
export type Turn = GalleryTurnView;
export type Artifact = GalleryArtifactView;
