export interface SectionNav {
  id: string;
  label: string;
}

export const sections: SectionNav[] = [
  { id: "addons", label: "Addons" },
  { id: "database", label: "Database" },
  { id: "downloads", label: "Downloads" },
  { id: "identity", label: "Identity" },
  { id: "libraries", label: "Libraries" },
  { id: "queue", label: "Queue" },
  { id: "settings", label: "Settings" },
  { id: "signaling", label: "Signaling" },
  { id: "smart-search", label: "Smart Search" },
  { id: "torrent", label: "Torrent" },
];
