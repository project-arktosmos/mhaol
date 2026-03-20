import type { MediaItem, MediaType } from "../../types";
import type { Ingestor } from "./ingestor";

const seenIds = new Set<string>();
const items: MediaItem[] = [];

function extractIdFromHref(href: string | null): string | null {
  if (!href) return null;
  const match = href.match(/\/detail\/([A-Z0-9]+)/);
  return match?.[1] ?? null;
}

function detectMediaType(el: Element): MediaType | null {
  // Check hero synopsis for "Season X・" prefix
  const card = el.closest('article[data-testid="top-hero-card"]');
  if (card) {
    const synopsis =
      card.querySelector('[data-testid="hero-synopsis"]')?.textContent ?? "";
    if (/^Season\s+\d+/i.test(synopsis)) return "tv";

    const playBtn = card.querySelector('[data-testid="play"]');
    const playLabel = playBtn?.getAttribute("aria-label") ?? "";
    if (/episode\s+\d+/i.test(playLabel)) return "tv";
  }

  // Check background image alt for "Season" suffix (e.g. "Young Sherlock - Season 1")
  const bgImg = el
    .closest("[data-animate-hero-background]")
    ?.querySelector("img[alt]");
  const alt = bgImg?.getAttribute("alt") ?? "";
  if (/- Season \d+$/i.test(alt)) return "tv";

  return null;
}

function resolveCategory(el: Element): string {
  // If inside the hero carousel
  if (el.closest('[data-testid="top-hero"]')) return "Featured";

  // Walk up to find a heading that labels a content row
  let node: Element | null = el;
  while (node) {
    const heading = node.querySelector(":scope > h2, :scope > h3");
    if (heading?.textContent?.trim()) return heading.textContent.trim();

    const prev = node.previousElementSibling;
    if (prev) {
      const h =
        prev.tagName === "H2" || prev.tagName === "H3"
          ? prev
          : prev.querySelector("h2, h3");
      if (h?.textContent?.trim()) return h.textContent.trim();
    }
    node = node.parentElement;
  }
  return "Unknown";
}

function addItem(
  title: string,
  id: string,
  el: Element,
  imageUrl?: string,
): void {
  if (seenIds.has(id)) return;
  seenIds.add(id);
  items.push({
    title,
    id,
    category: resolveCategory(el),
    mediaType: detectMediaType(el),
    source: "primevideo.com",
    imageUrl,
  });
}

function scanHeroCarousel(): void {
  const cards = document.querySelectorAll<HTMLElement>(
    'article[data-testid="top-hero-card"]',
  );
  for (const card of cards) {
    const titleEl = card.querySelector<HTMLElement>(
      'h2[data-testid="title-art"]',
    );
    const title = titleEl?.getAttribute("aria-label")?.trim();
    if (!title) continue;

    // Get ID from the image-link or the detail link inside the title
    const link =
      card.querySelector<HTMLAnchorElement>('a[data-testid="image-link"]') ??
      titleEl?.querySelector<HTMLAnchorElement>('a[href*="/detail/"]');
    const id = extractIdFromHref(link?.getAttribute("href") ?? null);
    if (!id) continue;

    // Get the hero background image
    const img = card.querySelector<HTMLImageElement>(
      '[data-animate-hero-background] img[data-testid="base-image"]',
    );
    addItem(title, id, card, img?.src);
  }
}

function scanAllDetailLinks(): void {
  const links = document.querySelectorAll<HTMLAnchorElement>(
    'a[href*="/detail/"]',
  );
  for (const link of links) {
    const id = extractIdFromHref(link.getAttribute("href"));
    if (!id || seenIds.has(id)) continue;

    // Try to resolve the title from multiple sources
    const title =
      link
        .getAttribute("aria-label")
        ?.replace(/^More details for /i, "")
        .trim() ||
      link
        .querySelector("img[alt]")
        ?.getAttribute("alt")
        ?.replace(/ - Season \d+$/i, "")
        .trim() ||
      link.textContent?.trim();
    if (!title || title.length < 2) continue;

    const img =
      link.querySelector<HTMLImageElement>("img") ??
      link
        .closest("article")
        ?.querySelector<HTMLImageElement>('img[data-testid="base-image"]');
    addItem(title, id, link, img?.src);
  }
}

export const primevideo: Ingestor = {
  source: "primevideo.com",

  matches(hostname: string): boolean {
    return (
      hostname.includes("primevideo.com") ||
      hostname.includes("amazon.com/gp/video")
    );
  },

  scan(): MediaItem[] {
    scanHeroCarousel();
    scanAllDetailLinks();
    return items;
  },
};
