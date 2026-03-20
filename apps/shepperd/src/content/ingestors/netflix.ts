import type { MediaItem } from "../../types";
import type { Ingestor } from "./ingestor";

const seenIds = new Set<string>();
const items: MediaItem[] = [];

function extractTrackingVideoId(el: Element): string | null {
  const ctx = el.getAttribute("data-ui-tracking-context");
  if (!ctx) return null;
  try {
    const parsed = JSON.parse(decodeURIComponent(ctx));
    return parsed.video_id ? String(parsed.video_id) : null;
  } catch {
    return null;
  }
}

function addFromLink(link: HTMLAnchorElement, category: string): void {
  const title = link.getAttribute("aria-label")?.trim();
  if (!title) return;

  const trackingEl = link.closest(".ptrack-content[data-ui-tracking-context]");
  const id = trackingEl ? extractTrackingVideoId(trackingEl) : null;
  if (!id || seenIds.has(id)) return;

  const img = link.querySelector<HTMLImageElement>("img.boxart-image");
  seenIds.add(id);
  items.push({
    title,
    id,
    category,
    mediaType: null,
    source: "netflix.com",
    imageUrl: img?.src,
  });
}

function scanBillboard(): void {
  const logoImg = document.querySelector<HTMLImageElement>(
    "div.billboard-title img.title-logo",
  );
  if (!logoImg) return;

  const title = logoImg.getAttribute("title")?.trim();
  if (!title) return;

  const trackingEl = logoImg.closest(
    ".ptrack-content[data-ui-tracking-context]",
  );
  const id = trackingEl ? extractTrackingVideoId(trackingEl) : null;
  if (!id || seenIds.has(id)) return;

  seenIds.add(id);
  items.push({
    title,
    id,
    category: "Featured",
    mediaType: null,
    source: "netflix.com",
    imageUrl: logoImg.src,
  });
}

function scanBrowseRows(): void {
  const rows = document.querySelectorAll(".lolomoRow");
  for (const row of rows) {
    const categoryEl = row.querySelector("h2.rowTitle");
    const category = categoryEl?.textContent?.trim() ?? "Unknown";
    const links = row.querySelectorAll<HTMLAnchorElement>(
      "a.slider-refocus[aria-label]",
    );
    for (const link of links) addFromLink(link, category);
  }
}

function scanMyList(): void {
  const gallery = document.querySelector(".galleryLockups");
  if (!gallery) return;
  const links = gallery.querySelectorAll<HTMLAnchorElement>(
    "a.slider-refocus[aria-label]",
  );
  for (const link of links) addFromLink(link, "My List");
}

export const netflix: Ingestor = {
  source: "netflix.com",

  matches(hostname: string): boolean {
    return hostname.includes("netflix.com");
  },

  scan(): MediaItem[] {
    scanBillboard();
    scanBrowseRows();
    scanMyList();
    return items;
  },
};
