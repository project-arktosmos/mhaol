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
};

export const load: PageLoad = async () => {
  if (!isTransportReady()) return EMPTY_MEDIA;
  try {
    return await fetchJson("/api/media");
  } catch (err) {
    return {
      ...EMPTY_MEDIA,
      error: err instanceof Error ? err.message : String(err),
    };
  }
};
