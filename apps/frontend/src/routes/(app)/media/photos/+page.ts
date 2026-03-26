import { fetchJson } from "ui-lib/transport/fetch-helpers";
import { isTransportReady } from "ui-lib/transport/transport-context";
import type { PageLoad } from "./$types";

const EMPTY_MEDIA = {
  mediaTypes: [],
  categories: [],
  linkSources: [],
  itemsByCategory: {},
  itemsByType: {},
  libraries: {},
  images: [],
};

export const load: PageLoad = async () => {
  if (!isTransportReady()) return EMPTY_MEDIA;
  try {
    const [media, images] = await Promise.all([
      fetchJson<Record<string, unknown>>("/api/media"),
      fetchJson<{ images: unknown[] }>("/api/images"),
    ]);
    return { ...media, images: images.images ?? [] };
  } catch (err) {
    return {
      ...EMPTY_MEDIA,
      error: err instanceof Error ? err.message : String(err),
    };
  }
};
