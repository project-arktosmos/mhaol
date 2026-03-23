import TorrentModalContent from "ui-lib/components/torrent/TorrentModalContent.svelte";
import DownloadsModalContent from "ui-lib/components/downloads/DownloadsModalContent.svelte";
import IdentityModalContent from "ui-lib/components/identity/IdentityModalContent.svelte";
import VideoSettingsModalContent from "ui-lib/components/settings/VideoSettingsModalContent.svelte";
import AddonsModalContent from "ui-lib/components/addons/AddonsModalContent.svelte";
import LlmModalContent from "ui-lib/components/llm/LlmModalContent.svelte";
import DatabaseModalContent from "ui-lib/components/database/DatabaseModalContent.svelte";
import SignalingInfoContent from "ui-lib/components/signaling/SignalingInfoContent.svelte";
import LibraryModalContent from "ui-lib/components/libraries/LibraryModalContent.svelte";
import QueueMonitorContent from "ui-lib/components/queue/QueueMonitorContent.svelte";
export type SectionEntry = {
  component: any;
  label: string;
  props?: Record<string, any>;
};

export const sectionComponents: Record<string, SectionEntry> = {
  addons: { component: AddonsModalContent, label: "Addons" },
  database: { component: DatabaseModalContent, label: "Database" },
  downloads: { component: DownloadsModalContent, label: "Downloads" },
  identity: { component: IdentityModalContent, label: "Identity" },
  libraries: { component: LibraryModalContent, label: "Libraries" },
  queue: { component: QueueMonitorContent, label: "Queue" },
  "smart-search": { component: LlmModalContent, label: "Smart Search" },
  settings: { component: VideoSettingsModalContent, label: "Settings" },
  signaling: { component: SignalingInfoContent, label: "Signaling" },
  torrent: { component: TorrentModalContent, label: "Torrent" },
};

export const sections = Object.entries(sectionComponents)
  .map(([id, entry]) => ({ id, label: entry.label }))
  .sort((a, b) => a.label.localeCompare(b.label));
