import { error } from "@sveltejs/kit";
import { getMediaConfig } from "ui-lib/data/media-registry";
import { fetchJson } from "ui-lib/transport/fetch-helpers";
import { isTransportReady } from "ui-lib/transport/transport-context";
import type { PageLoad } from "./$types";

const EMPTY_MEDIA = {
  mediaTypes: [],
  categories: [],
  linkSources: [],
  itemsByCategory: {},
  itemsByType: {},
  lists: [],
  libraries: {},
};

export const ssr = false;

export const load: PageLoad = async ({ params }) => {
  const config = getMediaConfig(params.slug);
  if (!config) throw error(404, "Not found");

  if (config.features.libraryItems && isTransportReady()) {
    try {
      const mediaData = await fetchJson("/api/media");
      return { config, mediaData };
    } catch {
      return { config, mediaData: EMPTY_MEDIA };
    }
  }

  return { config };
};
