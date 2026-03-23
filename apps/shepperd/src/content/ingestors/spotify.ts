import type { MediaItem } from "../../types";
import type { Ingestor } from "./ingestor";

const seenIds = new Set<string>();
const items: MediaItem[] = [];

function getLibraryGrid(): HTMLElement | null {
  return document.querySelector<HTMLElement>(
    '[role="grid"][aria-label="Your Library"]',
  );
}

function scanLibraryRows(
  uriType: string,
  mediaType: MediaItem["mediaType"],
  category: string,
  extractArtist?: (subtitleText: string) => string | undefined,
): void {
  const grid = getLibraryGrid();
  if (!grid) return;

  const rows = grid.querySelectorAll<HTMLElement>('[data-encore-id="listRow"]');
  for (const row of rows) {
    const labelledBy = row.getAttribute("aria-labelledby") ?? "";
    const match = labelledBy.match(
      new RegExp(`listrow-title-spotify:${uriType}:(\\S+)`),
    );
    if (!match) continue;

    const id = match[1];
    if (seenIds.has(id)) continue;

    const titleEl = row.querySelector<HTMLElement>(
      '[data-encore-id="listRowTitle"] .e-10180-line-clamp',
    );
    const title = titleEl?.textContent?.trim();
    if (!title) continue;

    const subtitleEl = row.querySelector<HTMLElement>(
      '[data-encore-id="listRowSubtitle"]',
    );
    const subtitleText = subtitleEl?.textContent?.trim() ?? "";
    const artist = extractArtist?.(subtitleText);

    const img = row.querySelector<HTMLImageElement>(
      '[data-testid="entity-image"]',
    );

    seenIds.add(id);
    items.push({
      title,
      id,
      category,
      mediaType,
      source: "open.spotify.com",
      artist,
      imageUrl: img?.src,
    });
  }
}

function scanLibraryPlaylists(): void {
  scanLibraryRows("playlist", "music", "Library", (subtitle) => {
    const match = subtitle.match(/Playlist\s*[•·]\s*(.+)/);
    return match?.[1]?.trim() || undefined;
  });
}

function scanLibraryArtists(): void {
  scanLibraryRows("artist", "music", "Library");
}

function scanLibraryAlbums(): void {
  scanLibraryRows("album", "music", "Library", (subtitle) => {
    const match = subtitle.match(
      /(?:Single|EP|Compilation|Album)\s*[•·]\s*(.+)/,
    );
    return match?.[1]?.trim() || subtitle || undefined;
  });
}

export const spotify: Ingestor = {
  source: "open.spotify.com",
  instructions:
    'To capture your library, open "Your Library" in the left sidebar. Filter by "Playlists", "Artists", or "Albums" using the tabs at the top. The list is virtualized — scroll through it slowly so all items load into view, then click Refresh.',

  matches(hostname: string): boolean {
    return hostname.includes("open.spotify.com");
  },

  scan(): MediaItem[] {
    scanLibraryPlaylists();
    scanLibraryArtists();
    scanLibraryAlbums();
    return items;
  },
};
