import type { McpServerStatus } from "../types/gallery";
import { isTauriRuntime } from "./tauriRuntime";

export async function getMcpServerStatus(): Promise<McpServerStatus> {
  if (!isTauriRuntime()) {
    return {
      running: false,
      url: null,
      port: null,
    };
  }

  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<McpServerStatus>("get_mcp_server_status");
}
