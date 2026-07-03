import type { McpServerStatus } from "../types/gallery";
import { isTauriRuntime } from "./tauriRuntime";

export async function getMcpServerStatus(): Promise<McpServerStatus> {
  if (!isTauriRuntime()) {
    return {
      running: false,
      status: "stopped",
      host: "127.0.0.1",
      url: null,
      port: null,
    };
  }

  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<McpServerStatus>("get_mcp_server_status");
}

export async function listenMcpStatusUpdates(onUpdate: (status: McpServerStatus) => void) {
  if (!isTauriRuntime()) {
    return () => {};
  }

  const { listen } = await import("@tauri-apps/api/event");
  return listen<McpServerStatus>("mcp-status-updated", (event) => {
    onUpdate(event.payload);
  });
}
