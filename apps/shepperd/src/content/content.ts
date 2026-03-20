import type { CatalogResponse } from "../types";
import { findIngestor } from "./ingestors";

const ingestor = findIngestor(location.hostname);
let scanTimer: ReturnType<typeof setTimeout> | null = null;

function scan(): CatalogResponse {
  return {
    items: ingestor?.scan() ?? [],
    source: ingestor?.source ?? null,
  };
}

function debouncedScan(): void {
  if (scanTimer) clearTimeout(scanTimer);
  scanTimer = setTimeout(scan, 200);
}

// Initial scan + observer only if we have a matching ingestor
if (ingestor) {
  scan();
  const observer = new MutationObserver(debouncedScan);
  observer.observe(document.body, { childList: true, subtree: true });
}

chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
  if (message.type === "GET_PAGE_INFO") {
    sendResponse({
      title: document.title,
      url: window.location.href,
    });
  } else if (message.type === "GET_CATALOG") {
    sendResponse(scan());
  }
  return true;
});
