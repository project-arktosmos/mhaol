import type { MediaItem } from "../../types";
import type { Ingestor } from "./ingestor";

const seenIds = new Set<string>();
const items: MediaItem[] = [];

function extractVideoId(el: Element): string | null {
  const ctx = el.getAttribute("data-ui-tracking-context");
  if (!ctx) return null;
  try {
    const parsed = JSON.parse(decodeURIComponent(ctx));
    return parsed.video_id ? String(parsed.video_id) : null;
  } catch {
    return null;
  }
}

function scanGallery(): void {
  const gallery = document.querySelector(".galleryLockups");
  if (!gallery) return;

  const trackingEls = gallery.querySelectorAll(
    ".ptrack-content[data-ui-tracking-context]",
  );
  for (const el of trackingEls) {
    const id = extractVideoId(el);
    if (!id || seenIds.has(id)) continue;

    const link = el.querySelector<HTMLAnchorElement>(
      "a.slider-refocus[aria-label]",
    );
    const title = link?.getAttribute("aria-label")?.trim();
    if (!title) continue;

    const img = el.querySelector<HTMLImageElement>("img.boxart-image");

    seenIds.add(id);
    items.push({
      title,
      id,
      category: "My List",
      mediaType: null,
      source: "netflix.com",
      imageUrl: img?.src,
    });
  }
}

export const netflix: Ingestor = {
  source: "netflix.com",
  instructions:
    'To capture your saved titles, go to "My List" — click the hamburger menu (\u2630) in the top-left corner, then select "My List". You can also go directly to netflix.com/browse/my-list. Scroll down to load all titles, then click Refresh.',

  matches(hostname: string): boolean {
    return hostname.includes("netflix.com");
  },

  scan(): MediaItem[] {
    scanGallery();
    return items;
  },
};
