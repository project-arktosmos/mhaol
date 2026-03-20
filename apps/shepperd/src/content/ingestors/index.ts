import type { Ingestor } from "./ingestor";
import { netflix } from "./netflix";
import { primevideo } from "./primevideo";
import { spotify } from "./spotify";
import { hbomax } from "./hbomax";

const ingestors: Ingestor[] = [netflix, primevideo, spotify, hbomax];

export function findIngestor(hostname: string): Ingestor | null {
  return ingestors.find((i) => i.matches(hostname)) ?? null;
}
