import type { MediaItem, MediaType } from "../../types";
import type { Ingestor } from "./ingestor";

const seenIds = new Set<string>();
const items: MediaItem[] = [];

function resolveType(tile: HTMLAnchorElement): MediaType | null {
  const sonicType = tile.getAttribute("data-sonic-type");
  if (sonicType === "show") return "tv";

  const href = tile.getAttribute("href") ?? "";
  if (href.startsWith("/movie/")) return "movies";
  if (href.startsWith("/show/") || href.startsWith("/mini-series/"))
    return "tv";

  return null;
}

function scanHero(): void {
  const heroBtn = document.querySelector<HTMLAnchorElement>(
    'a[data-testid="immersive-hero-surface-button"]',
  );
  if (!heroBtn) return;

  const label = heroBtn.getAttribute("aria-label") ?? "";
  // aria-label like "Rooster, Ir a Series"
  const title = label.split(",")[0]?.trim();
  if (!title) return;

  // Use the title as a pseudo-ID since the hero doesn't expose a sonic-id
  const id = title.toLowerCase().replace(/\s+/g, "-");
  if (seenIds.has(id)) return;

  const heroSection = heroBtn.closest("section");
  const img = heroSection?.querySelector<HTMLImageElement>("img[alt]");
  const seasons = heroSection?.querySelector(
    '[data-testid="metadata_total_seasons_hero"]',
  );
  const mediaType: MediaType | null = seasons ? "tv" : null;

  seenIds.add(id);
  items.push({
    title,
    id,
    category: "Featured",
    mediaType,
    source: "play.hbomax.com",
    imageUrl: img?.src,
  });
}

function scanTiles(): void {
  const tiles = document.querySelectorAll<HTMLAnchorElement>(
    'a[data-sonic-id][data-testid$="_tile"]',
  );
  for (const tile of tiles) {
    const id = tile.getAttribute("data-sonic-id");
    if (!id || seenIds.has(id)) continue;

    // Title from backup text inside the tile
    const backupText = tile.querySelector("p")?.textContent?.trim();
    if (!backupText) continue;

    const img = tile.querySelector<HTMLImageElement>("img");

    // Category from parent section's heading
    const section = tile.closest('section[data-testid$="_rail"]');
    const heading = section?.querySelector("h2 span")?.textContent?.trim();
    const category = heading ?? "Browse";

    seenIds.add(id);
    items.push({
      title: backupText,
      id,
      category,
      mediaType: resolveType(tile),
      source: "play.hbomax.com",
      imageUrl: img?.src,
    });
  }
}

export const hbomax: Ingestor = {
  source: "play.hbomax.com",
  instructions:
    'To capture your saved titles, click "My List" from the top navigation menu. Scrolling the homepage will also capture titles from each content row.',

  matches(hostname: string): boolean {
    return hostname.includes("play.hbomax.com");
  },

  scan(): MediaItem[] {
    scanHero();
    scanTiles();
    return items;
  },
};
