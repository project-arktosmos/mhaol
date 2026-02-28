const YOUTUBE_CHANNEL_PATTERNS = [
  // /channel/UCxxxx - direct channel ID
  /youtube\.com\/channel\/(UC[a-zA-Z0-9_-]+)/,
  // /@handle
  /youtube\.com\/(@[\w.-]+)/,
  // /c/CustomName
  /youtube\.com\/c\/([\w.-]+)/,
  // /user/Username
  /youtube\.com\/user\/([\w.-]+)/,
];

/** Check if a URL is a YouTube channel URL */
export function isChannelUrl(url: string): boolean {
  return YOUTUBE_CHANNEL_PATTERNS.some((pattern) => pattern.test(url));
}

/** Extract channel ID directly from a /channel/UCxxx URL, or null if not that format */
export function extractChannelId(url: string): string | null {
  const match = url.match(/youtube\.com\/channel\/(UC[a-zA-Z0-9_-]+)/);
  return match ? match[1] : null;
}

/**
 * Resolve any YouTube channel URL to a channel ID.
 * For /channel/UCxxx URLs, extracts directly.
 * For @handle, /c/, /user/ URLs, fetches the page and finds the canonical channel ID.
 */
export async function resolveChannelId(url: string): Promise<string> {
  // Try direct extraction first
  const directId = extractChannelId(url);
  if (directId) return directId;

  if (!isChannelUrl(url)) {
    throw new Error("Not a valid YouTube channel URL");
  }

  // Normalize URL to have https://
  let normalizedUrl = url.trim();
  if (!normalizedUrl.startsWith("http")) {
    normalizedUrl = `https://${normalizedUrl}`;
  }

  // Fetch the YouTube page and extract channel ID from canonical link
  const response = await fetch(normalizedUrl, {
    headers: {
      "User-Agent":
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
    },
    redirect: "follow",
  });

  if (!response.ok) {
    throw new Error(`Failed to fetch YouTube page: ${response.status}`);
  }

  const html = await response.text();

  // YouTube pages include the channel ID in several places:
  // <link rel="canonical" href="https://www.youtube.com/channel/UCxxxxxx">
  // <meta property="og:url" content="https://www.youtube.com/channel/UCxxxxxx">
  // "channelId":"UCxxxxxx"
  const patterns = [
    /youtube\.com\/channel\/(UC[a-zA-Z0-9_-]+)/,
    /"channelId"\s*:\s*"(UC[a-zA-Z0-9_-]+)"/,
    /"externalId"\s*:\s*"(UC[a-zA-Z0-9_-]+)"/,
  ];

  for (const pattern of patterns) {
    const match = html.match(pattern);
    if (match) return match[1];
  }

  throw new Error("Could not resolve channel ID from the YouTube page");
}

/** Build the YouTube RSS feed URL for a channel ID */
export function getRssFeedUrl(channelId: string): string {
  return `https://www.youtube.com/feeds/videos.xml?channel_id=${channelId}`;
}
