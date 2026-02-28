import { getRssFeedUrl } from "./channel.js";

export interface YouTubeRssEntry {
  videoId: string;
  title: string;
  published: string;
  updated: string;
  thumbnailUrl: string;
  description: string;
  viewCount: number;
}

export interface YouTubeRssFeed {
  channelId: string;
  channelTitle: string;
  channelUrl: string;
  entries: YouTubeRssEntry[];
}

/** Extract text content between XML tags */
function extractTag(xml: string, tag: string): string {
  const match = xml.match(new RegExp(`<${tag}[^>]*>([\\s\\S]*?)<\\/${tag}>`));
  return match ? match[1].trim() : "";
}

/** Extract an attribute value from an XML tag */
function extractAttr(xml: string, tag: string, attr: string): string {
  const match = xml.match(new RegExp(`<${tag}[^>]*\\s${attr}="([^"]*)"`, "i"));
  return match ? match[1] : "";
}

/** Parse a single <entry> block from the Atom feed */
function parseEntry(entryXml: string): YouTubeRssEntry {
  const videoId = extractTag(entryXml, "yt:videoId");
  const title = decodeXmlEntities(extractTag(entryXml, "title"));
  const published = extractTag(entryXml, "published");
  const updated = extractTag(entryXml, "updated");
  const thumbnailUrl = extractAttr(entryXml, "media:thumbnail", "url");
  const description = decodeXmlEntities(
    extractTag(entryXml, "media:description"),
  );
  const viewsStr = extractAttr(entryXml, "media:statistics", "views");
  const viewCount = viewsStr ? parseInt(viewsStr, 10) : 0;

  return {
    videoId,
    title,
    published,
    updated,
    thumbnailUrl,
    description,
    viewCount,
  };
}

/** Decode common XML entities */
function decodeXmlEntities(text: string): string {
  return text
    .replace(/&amp;/g, "&")
    .replace(/&lt;/g, "<")
    .replace(/&gt;/g, ">")
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'")
    .replace(/&apos;/g, "'");
}

/** Fetch and parse a YouTube channel's RSS feed */
export async function fetchYouTubeRssFeed(
  channelId: string,
): Promise<YouTubeRssFeed> {
  const feedUrl = getRssFeedUrl(channelId);

  const response = await fetch(feedUrl);
  if (!response.ok) {
    throw new Error(`Failed to fetch RSS feed: ${response.status}`);
  }

  const xml = await response.text();

  const channelTitle = decodeXmlEntities(extractTag(xml, "title"));
  const channelUrl =
    extractAttr(xml, "link", "href") ||
    `https://www.youtube.com/channel/${channelId}`;

  // Split on <entry> blocks
  const entryBlocks = xml.split("<entry>").slice(1);
  const entries = entryBlocks.map((block) => parseEntry("<entry>" + block));

  return {
    channelId,
    channelTitle,
    channelUrl,
    entries,
  };
}
