import { fetchJson } from "ui-lib/transport/fetch-helpers";
import type { PageLoad } from "./$types";

export const load: PageLoad = async () => {
  try {
    return await fetchJson("/api/media");
  } catch (err) {
    return {
      error: err instanceof Error ? err.message : String(err),
    };
  }
};
