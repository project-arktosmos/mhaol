export type MediaType = "movies" | "tv" | "music";

export interface MediaItem {
  title: string;
  id: string;
  category: string;
  mediaType: MediaType | null;
  source: string;
  artist?: string;
  imageUrl?: string;
}

export interface CatalogResponse {
  items: MediaItem[];
  source: string | null;
  instructions: string | null;
}
