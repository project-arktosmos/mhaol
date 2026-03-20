import type { MediaItem } from "../../types";

export interface Ingestor {
  source: string;
  matches(hostname: string): boolean;
  scan(): MediaItem[];
}
