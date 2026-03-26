import { fetchJson } from "ui-lib/transport/fetch-helpers";
import type { PageLoad } from "./$types";

export const load: PageLoad = async () => {
  try {
    return await fetchJson("/api/media");
  } catch (err) {
    return {
      mediaTypes: [],
      categories: [],
      linkSources: [],
      itemsByCategory: {},
      itemsByType: {},
      libraries: {},
      error: err instanceof Error ? err.message : String(err),
    };
  }
};
