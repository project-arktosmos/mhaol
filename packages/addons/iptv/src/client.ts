import type {
	IptvOrgChannel,
	IptvOrgStream,
	IptvOrgLogo,
	IptvOrgCategory,
	IptvOrgCountry,
	IptvOrgGuide,
	DisplayIptvChannel,
	DisplayIptvStream,
	DisplayIptvSearchResult,
	IptvSearchOptions
} from './types.js';

const BASE = 'https://iptv-org.github.io/api';
const CACHE_TTL_MS = 60 * 60 * 1000; // 1 hour

interface IptvCache {
	channels: DisplayIptvChannel[];
	streamsByChannel: Map<string, DisplayIptvStream[]>;
	categories: IptvOrgCategory[];
	countries: IptvOrgCountry[];
	fetchedAt: number;
}

let cache: IptvCache | null = null;
let inFlight: Promise<IptvCache> | null = null;

async function fetchJson<T>(path: string): Promise<T> {
	const res = await fetch(`${BASE}/${path}`, { headers: { Accept: 'application/json' } });
	if (!res.ok) throw new Error(`iptv-org ${path}: HTTP ${res.status}`);
	return (await res.json()) as T;
}

function buildLogoMap(logos: IptvOrgLogo[]): Map<string, string> {
	const map = new Map<string, string>();
	for (const logo of logos) {
		if (!logo.channel || !logo.url) continue;
		// Prefer logos without a feed binding (canonical channel logo)
		if (!map.has(logo.channel) || !logo.feed) map.set(logo.channel, logo.url);
	}
	return map;
}

function buildEpgSet(guides: IptvOrgGuide[]): Set<string> {
	const set = new Set<string>();
	for (const g of guides) {
		if (g.channel) set.add(g.channel);
	}
	return set;
}

async function loadCache(): Promise<IptvCache> {
	if (cache && Date.now() - cache.fetchedAt < CACHE_TTL_MS) return cache;
	if (inFlight) return inFlight;

	inFlight = (async () => {
		const [channels, streams, logos, categories, countries, guides] = await Promise.all([
			fetchJson<IptvOrgChannel[]>('channels.json'),
			fetchJson<IptvOrgStream[]>('streams.json'),
			fetchJson<IptvOrgLogo[]>('logos.json'),
			fetchJson<IptvOrgCategory[]>('categories.json'),
			fetchJson<IptvOrgCountry[]>('countries.json'),
			fetchJson<IptvOrgGuide[]>('guides.json').catch(() => [] as IptvOrgGuide[])
		]);

		const logoMap = buildLogoMap(logos);
		const epgSet = buildEpgSet(guides);

		const streamsByChannel = new Map<string, DisplayIptvStream[]>();
		for (const s of streams) {
			if (!s.channel || !s.url) continue;
			const list = streamsByChannel.get(s.channel) ?? [];
			list.push({
				channel: s.channel,
				url: s.url,
				httpReferrer: s.referrer ?? null,
				userAgent: s.user_agent ?? null,
				quality: s.quality ?? null
			});
			streamsByChannel.set(s.channel, list);
		}

		const displayChannels: DisplayIptvChannel[] = [];
		for (const ch of channels) {
			if (!streamsByChannel.has(ch.id)) continue;
			displayChannels.push({
				id: ch.id,
				name: ch.name,
				country: ch.country,
				categories: ch.categories ?? [],
				logo: logoMap.get(ch.id) ?? null,
				website: ch.website ?? null,
				isNsfw: Boolean(ch.is_nsfw),
				hasEpg: epgSet.has(ch.id)
			});
		}

		const next: IptvCache = {
			channels: displayChannels,
			streamsByChannel,
			categories,
			countries,
			fetchedAt: Date.now()
		};
		cache = next;
		return next;
	})();

	try {
		return await inFlight;
	} finally {
		inFlight = null;
	}
}

function applyFilter(
	channels: DisplayIptvChannel[],
	query: string,
	opts: IptvSearchOptions
): DisplayIptvChannel[] {
	const q = query.trim().toLowerCase();
	const category = opts.category?.trim().toLowerCase() ?? '';
	const country = opts.country?.trim().toLowerCase() ?? '';
	const hasEpgOnly = opts.hasEpg === true;

	return channels.filter((ch) => {
		if (q && !ch.name.toLowerCase().includes(q)) return false;
		if (category && !ch.categories.some((c) => c.toLowerCase() === category)) return false;
		if (country && ch.country.toLowerCase() !== country) return false;
		if (hasEpgOnly && !ch.hasEpg) return false;
		return true;
	});
}

export async function searchChannels(
	query: string,
	options: IptvSearchOptions = {}
): Promise<DisplayIptvSearchResult> {
	const data = await loadCache();
	const filtered = applyFilter(data.channels, query, options);
	const page = Math.max(1, options.page ?? 1);
	const limit = Math.min(200, Math.max(1, options.limit ?? 50));
	const start = (page - 1) * limit;
	return {
		channels: filtered.slice(start, start + limit),
		total: filtered.length,
		page,
		limit
	};
}

export async function getChannel(id: string): Promise<DisplayIptvChannel | null> {
	const data = await loadCache();
	return data.channels.find((ch) => ch.id === id) ?? null;
}

export async function getStreams(channelId: string): Promise<DisplayIptvStream[]> {
	const data = await loadCache();
	return data.streamsByChannel.get(channelId) ?? [];
}

export async function getCategories(): Promise<IptvOrgCategory[]> {
	const data = await loadCache();
	return data.categories;
}

export async function getCountries(): Promise<IptvOrgCountry[]> {
	const data = await loadCache();
	return data.countries;
}

export function clearCache(): void {
	cache = null;
}
