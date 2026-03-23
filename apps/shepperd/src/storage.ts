import type { MediaItem } from "./types";

const STORAGE_KEY = "shepperd_catalog";

function itemKey(item: MediaItem): string {
  return `${item.source}::${item.id}`;
}

export async function loadItems(): Promise<MediaItem[]> {
  const result = await chrome.storage.local.get(STORAGE_KEY);
  return result[STORAGE_KEY] ?? [];
}

export async function mergeItems(newItems: MediaItem[]): Promise<MediaItem[]> {
  const existing = await loadItems();
  const seen = new Set(existing.map(itemKey));
  const added: MediaItem[] = [];

  for (const item of newItems) {
    if (!seen.has(itemKey(item))) {
      seen.add(itemKey(item));
      added.push(item);
    }
  }

  if (added.length > 0) {
    const merged = [...added, ...existing];
    await chrome.storage.local.set({ [STORAGE_KEY]: merged });
    return merged;
  }

  return existing;
}

export async function clearItems(): Promise<void> {
  await chrome.storage.local.remove(STORAGE_KEY);
}
