import { error } from "@sveltejs/kit";
import { getMusicConfig } from "ui-lib/data/media-registry";
import type { PageLoad } from "./$types";

export const ssr = false;

export const load: PageLoad = async ({ params }) => {
  const config = getMusicConfig(params.subslug);
  if (!config) throw error(404, "Not found");
  return { config };
};
