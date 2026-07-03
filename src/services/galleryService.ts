import { mockGalleryView } from "../data/mockSessions";
import type { GalleryView } from "../types/gallery";
import { isTauriRuntime } from "./tauriRuntime";

export async function listGalleryView(): Promise<GalleryView> {
  if (!isTauriRuntime()) {
    return mockGalleryView;
  }

  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<GalleryView>("list_gallery_view");
}

export async function listenGalleryUpdates(onUpdate: (view: GalleryView) => void) {
  if (!isTauriRuntime()) {
    return () => {};
  }

  const { listen } = await import("@tauri-apps/api/event");
  return listen<GalleryView>("gallery-updated", (event) => {
    onUpdate(event.payload);
  });
}
