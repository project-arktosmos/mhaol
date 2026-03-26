import { fetchJson } from "ui-lib/transport/fetch-helpers";
import type { PageLoad } from "./$types";

export const load: PageLoad = async () => {
  try {
    const [media, images] = await Promise.all([
      fetchJson<Record<string, unknown>>("/api/media"),
      fetchJson<{ images: unknown[] }>("/api/images"),
    ]);
    return { ...media, images: images.images ?? [] };
  } catch (err) {
    return {
      error: err instanceof Error ? err.message : String(err),
    };
  }
};
