export interface SettingRow {
  key: string;
  value: string;
  created_at: string;
  updated_at: string;
}

export interface MetadataRow {
  key: string;
  value: string;
  type: "string" | "number" | "boolean" | "json";
  created_at: string;
  updated_at: string;
}

export interface DatabaseConfig {
  /** Path to the .sqlite file. Defaults to packages/database/mhaol.db */
  dbPath?: string;
  /** Whether to enable WAL mode. Defaults to true. */
  walMode?: boolean;
}

export interface YouTubeDownloadRow {
  download_id: string;
  url: string;
  video_id: string;
  title: string;
  state: string;
  progress: number;
  downloaded_bytes: number;
  total_bytes: number;
  output_path: string | null;
  error: string | null;
  mode: string;
  quality: string;
  format: string;
  video_quality: string | null;
  video_format: string | null;
  thumbnail_url: string | null;
  duration_seconds: number | null;
  created_at: string;
  updated_at: string;
}

export interface TorrentDownloadRow {
  info_hash: string;
  name: string;
  size: number;
  progress: number;
  state: string;
  download_speed: number;
  upload_speed: number;
  peers: number;
  seeds: number;
  added_at: number;
  eta: number | null;
  output_path: string | null;
  source: string;
  created_at: string;
  updated_at: string;
}

export interface LibraryRow {
  id: string;
  name: string;
  path: string;
  media_types: string;
  date_added: number;
  created_at: string;
  updated_at: string;
}

export interface MediaTypeRow {
  id: string;
  label: string;
}

export interface CategoryRow {
  id: string;
  media_type_id: string;
  label: string;
}

export interface LibraryItemRow {
  id: string;
  library_id: string;
  path: string;
  extension: string;
  media_type: string;
  category_id: string | null;
  created_at: string;
  updated_at: string;
}

export interface LibraryItemLinkRow {
  id: string;
  library_item_id: string;
  service: string;
  service_id: string;
  season_number: number | null;
  episode_number: number | null;
  created_at: string;
}

export interface LinkSourceRow {
  id: string;
  plugin: string;
  service: string;
  label: string;
  media_type_id: string;
  category_id: string | null;
}

export interface ImageTagRow {
  id: string;
  library_item_id: string;
  tag: string;
  score: number;
  created_at: string;
}
