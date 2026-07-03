import { mockGalleryView } from "../data/mockSessions";
import type { GalleryView, SidebarMode } from "../types/gallery";
import { isTauriRuntime } from "./tauriRuntime";

export async function listGalleryView(): Promise<GalleryView> {
  if (!isTauriRuntime()) {
    return mockGalleryView;
  }

  const { invoke } = await import("@tauri-apps/api/core");
  return withAssetUrls(await invoke<GalleryView>("list_gallery_view"));
}

export async function setSidebarMode(mode: SidebarMode): Promise<GalleryView> {
  if (!isTauriRuntime()) {
    return { ...mockGalleryView, sidebarMode: mode };
  }

  const { invoke } = await import("@tauri-apps/api/core");
  return withAssetUrls(await invoke<GalleryView>("set_sidebar_mode", { mode }));
}

export async function selectSession(sessionId: string): Promise<GalleryView> {
  if (!isTauriRuntime()) {
    return { ...mockGalleryView, selectedSessionId: sessionId };
  }

  const { invoke } = await import("@tauri-apps/api/core");
  return withAssetUrls(await invoke<GalleryView>("select_session", { sessionId }));
}

export async function toggleTurnCollapsed(sessionId: string, turnId: string): Promise<GalleryView> {
  if (!isTauriRuntime()) {
    return mockGalleryView;
  }

  const { invoke } = await import("@tauri-apps/api/core");
  return withAssetUrls(await invoke<GalleryView>("toggle_turn_collapsed", { sessionId, turnId }));
}

export async function listenGalleryUpdates(onUpdate: (view: GalleryView) => void) {
  if (!isTauriRuntime()) {
    return () => {};
  }

  const { listen } = await import("@tauri-apps/api/event");
  return listen<GalleryView>("gallery-updated", async (event) => {
    onUpdate(await withAssetUrls(event.payload));
  });
}

async function withAssetUrls(view: GalleryView): Promise<GalleryView> {
  if (!isTauriRuntime()) {
    return view;
  }

  const { convertFileSrc } = await import("@tauri-apps/api/core");
  return {
    ...view,
    sessions: view.sessions.map((session) => ({
      ...session,
      turns: session.turns.map((turn) => ({
        ...turn,
        artifacts: turn.artifacts.map((artifact) => ({
          ...artifact,
          assetUrl:
            artifact.localFilePath && artifact.status === "finished"
              ? convertFileSrc(artifact.localFilePath)
              : artifact.assetUrl,
        })),
      })),
    })),
  };
}
