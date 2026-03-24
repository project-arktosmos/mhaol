import { apiUrl } from "ui-lib/lib/api-base";
import type { PageLoad } from "./$types";

export const load: PageLoad = async ({ fetch }) => {
  const [mediaRes, imagesRes] = await Promise.all([
    fetch(apiUrl("/api/media")),
    fetch(apiUrl("/api/images")),
  ]);
  const media = await mediaRes.json();
  const images = await imagesRes.json();
  return { ...media, images: images.images ?? [] };
};
