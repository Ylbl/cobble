export type ArtifactKind = "pdf" | "png" | "svg" | "latex";

export type ArtifactStatus = "finished" | "compiling" | "failed";

export type ArtifactPreviewType = "large" | "small";

export type Artifact = {
  id: string;
  title: string;
  kind: ArtifactKind;
  status: ArtifactStatus;
  previewType: ArtifactPreviewType;
};

export type Turn = {
  id: string;
  index: number;
  hint: string;
  collapsed: boolean;
  artifacts: Artifact[];
};

export type Session = {
  id: string;
  title: string;
  source: "ZCode" | "Codex" | "Cursor";
  projectName: string;
  projectPath: string;
  updatedAt: string;
  artifactCount: number;
  turns: Turn[];
};
