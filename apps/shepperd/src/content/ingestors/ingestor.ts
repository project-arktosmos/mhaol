import type { MediaItem } from "../../types";

export interface Ingestor {
  source: string;
  instructions: string;
  matches(hostname: string): boolean;
  scan(): MediaItem[];
}
