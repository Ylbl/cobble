import type { GalleryView, LatexDoctorReport, McpServerStatus, SidecarConfigView } from "../types/gallery";
import { isTauriRuntime } from "./tauriRuntime";

export async function getSidecarConfig(): Promise<SidecarConfigView> {
  if (!isTauriRuntime()) {
    return defaultConfigView();
  }
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<SidecarConfigView>("get_sidecar_config");
}

export async function updateSidecarConfig(view: SidecarConfigView): Promise<SidecarConfigView> {
  if (!isTauriRuntime()) {
    return view;
  }
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<SidecarConfigView>("update_sidecar_config", { config: view.config });
}

export async function restartMcpServer(): Promise<McpServerStatus> {
  if (!isTauriRuntime()) {
    return { running: false, status: "stopped", host: "127.0.0.1", port: 39333, url: null };
  }
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<McpServerStatus>("restart_mcp_server");
}

export async function runLatexEnvironmentCheck(): Promise<LatexDoctorReport> {
  if (!isTauriRuntime()) {
    return {
      checkedAt: new Date().toISOString(),
      defaultEngine: "xelatex",
      tools: [],
      packages: [],
      smokeTest: null,
    };
  }
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<LatexDoctorReport>("run_latex_environment_check");
}

export async function runLatexSmokeTest(): Promise<GalleryView> {
  if (!isTauriRuntime()) {
    throw new Error("LaTeX smoke test requires Tauri runtime");
  }
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<GalleryView>("run_latex_smoke_test");
}

export async function openPath(path: string): Promise<void> {
  if (!isTauriRuntime()) {
    return;
  }
  const { invoke } = await import("@tauri-apps/api/core");
  await invoke("open_path", { path });
}

function defaultConfigView(): SidecarConfigView {
  const cwd = "dev-instance";
  return {
    config: {
      instanceName: "Sidecar",
      mcp: { host: "127.0.0.1", port: 39333 },
      latex: { engine: "xelatex", compileTimeoutSeconds: 60 },
      gallery: { defaultSidebarMode: "groups" },
      paths: { dataDir: "./data" },
    },
    instanceDir: cwd,
    configPath: `${cwd}/sidecar.config.json`,
    dataDir: `${cwd}/data`,
    galleryStatePath: `${cwd}/data/gallery-state.json`,
    galleryEventsPath: `${cwd}/data/gallery-events.jsonl`,
    artifactsDir: `${cwd}/data/artifacts`,
    logsDir: `${cwd}/data/logs`,
    debugArtifactsDir: `${cwd}/data/debug-artifacts`,
    lockPath: `${cwd}/data/.sidecar.lock`,
  };
}
