import { isTauriRuntime } from "./tauriRuntime";

export type PdfAssetDiagnostic = {
  artifactId: string;
  pdfUrl: string;
  localFilePath: string;
  localFileExists: boolean;
  isFile: boolean;
  fileSizeBytes?: number | null;
  canonicalFilePath?: string | null;
  dataDir: string;
  instanceDir: string;
  exePath?: string | null;
  exeDir?: string | null;
  underDataDir: boolean;
  underInstanceDir: boolean;
  configuredAssetScope: string;
  errorMessage?: string | null;
};

export async function readTextFile(path: string): Promise<string> {
  if (!isTauriRuntime()) {
    return "";
  }

  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<string>("read_text_file", { path });
}

export async function readBinaryFile(path: string): Promise<Uint8Array> {
  const { invoke } = await import("@tauri-apps/api/core");
  const bytes = await invoke<number[] | Uint8Array>("read_binary_file", { path });
  return bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes);
}

export async function recordPdfjsPreviewRequested(artifactId: string, pdfUrl: string): Promise<void> {
  const { invoke } = await import("@tauri-apps/api/core");
  await invoke("record_pdfjs_preview_requested", { artifactId, pdfUrl });
}

export async function diagnosePdfAssetPath(
  artifactId: string,
  localFilePath: string,
  pdfUrl: string,
): Promise<PdfAssetDiagnostic | null> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<PdfAssetDiagnostic>("diagnose_pdf_asset_path", { artifactId, localFilePath, pdfUrl });
}
