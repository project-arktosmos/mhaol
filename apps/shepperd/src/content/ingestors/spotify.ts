import type { MediaItem } from "../../types";
import type { Ingestor } from "./ingestor";

const seenIds = new Set<string>();
const items: MediaItem[] = [];

function scanCards(): void {
  const cards = document.querySelectorAll<HTMLElement>(
    '[data-encore-id="card"]',
  );
  for (const card of cards) {
    const link = card.querySelector<HTMLAnchorElement>(
      'a[href*="/playlist/"], a[href*="/album/"]',
    );
    if (!link) continue;

    const href = link.getAttribute("href") ?? "";
    const playlistMatch = href.match(/\/playlist\/([A-Za-z0-9]+)/);
    const albumMatch = href.match(/\/album\/([A-Za-z0-9]+)/);
    const id = playlistMatch?.[1] ?? albumMatch?.[1];
    const mediaType = playlistMatch ? "playlist" : "album";
    if (!id || seenIds.has(id)) continue;

    const titleEl = card.querySelector<HTMLElement>(
      'p[data-encore-id="cardTitle"]',
    );
    const title =
      titleEl?.getAttribute("title")?.trim() ?? titleEl?.textContent?.trim();
    if (!title) continue;

    const img = card.querySelector<HTMLImageElement>(
      'img[data-testid="card-image"]',
    );
    const shelf = card.closest("section[aria-label]");
    const category = shelf?.getAttribute("aria-label")?.trim() ?? "Library";

    seenIds.add(id);
    items.push({
      title,
      id,
      category,
      mediaType,
      source: "open.spotify.com",
      imageUrl: img?.src,
    });
  }
}

function scanTracks(): void {
  const grid = document.querySelector<HTMLElement>(
    '[data-testid="playlist-tracklist"]',
  );
  const playlistName = grid?.getAttribute("aria-label")?.trim() ?? "Tracks";

  const rows = document.querySelectorAll<HTMLElement>(
    '[data-testid="tracklist-row"]',
  );
  for (const row of rows) {
    const trackLink = row.querySelector<HTMLAnchorElement>(
      'a[data-testid="internal-track-link"]',
    );
    if (!trackLink) continue;

    const href = trackLink.getAttribute("href") ?? "";
    const match = href.match(/\/track\/([A-Za-z0-9]+)/);
    const id = match?.[1];
    if (!id || seenIds.has(id)) continue;

    const title = trackLink.textContent?.trim();
    if (!title) continue;

    // Extract artist names from artist links
    const artistLinks = row.querySelectorAll<HTMLAnchorElement>(
      'a[href*="/artist/"]',
    );
    const artist = Array.from(artistLinks)
      .map((a) => a.textContent?.trim())
      .filter(Boolean)
      .join(", ");

    const img = row.querySelector<HTMLImageElement>("img");

    seenIds.add(id);
    items.push({
      title,
      id,
      category: playlistName,
      mediaType: "song",
      source: "open.spotify.com",
      artist: artist || undefined,
      imageUrl: img?.src,
    });
  }
}

export const spotify: Ingestor = {
  source: "open.spotify.com",

  matches(hostname: string): boolean {
    return hostname.includes("open.spotify.com");
  },

  scan(): MediaItem[] {
    scanCards();
    scanTracks();
    return items;
  },
};
