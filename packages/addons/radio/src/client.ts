import type {
	RadioBrowserStation,
	RadioBrowserTag,
	RadioBrowserCountry,
	RadioBrowserLanguage,
	DisplayRadioStation,
	DisplayRadioSearchResult,
	RadioSearchOptions
} from './types.js';

// Radio Browser exposes multiple mirrors. We default to one of the load-balanced
// hosts; consumers can override with `setRadioBrowserBase()` if they want to
// pin a specific mirror.
const DEFAULT_BASE = 'https://de1.api.radio-browser.info/json';
const HEADERS = {
	Accept: 'application/json',
	'User-Agent': 'Mhaol/0.0.1 (https://github.com/project-arktosmos/mhaol)'
};

let baseUrl = DEFAULT_BASE;

export function setRadioBrowserBase(url: string): void {
	baseUrl = url.replace(/\/+$/, '');
}

async function rbFetch<T>(path: string, params?: Record<string, string>): Promise<T> {
	const search = params ? `?${new URLSearchParams(params).toString()}` : '';
	const res = await fetch(`${baseUrl}${path}${search}`, { headers: HEADERS });
	if (!res.ok) throw new Error(`radio-browser ${path}: HTTP ${res.status}`);
	return (await res.json()) as T;
}

function splitTags(tags: string): string[] {
	return tags
		.split(',')
		.map((t) => t.trim())
		.filter((t) => t.length > 0);
}

function toDisplayStation(s: RadioBrowserStation): DisplayRadioStation {
	const streamUrl = s.url_resolved || s.url;
	return {
		id: s.stationuuid,
		name: s.name.trim(),
		streamUrl,
		homepage: s.homepage || null,
		logo: s.favicon || null,
		tags: splitTags(s.tags ?? ''),
		country: s.country ?? '',
		countryCode: s.countrycode ?? '',
		language: s.language ?? '',
		codec: s.codec || null,
		bitrate: s.bitrate > 0 ? s.bitrate : null,
		isHls: s.hls === 1,
		votes: s.votes ?? 0,
		clickCount: s.clickcount ?? 0
	};
}

export async function searchStations(
	query: string,
	options: RadioSearchOptions = {}
): Promise<DisplayRadioSearchResult> {
	const page = Math.max(1, options.page ?? 1);
	const limit = Math.min(200, Math.max(1, options.limit ?? 50));
	const offset = (page - 1) * limit;

	const params: Record<string, string> = {
		limit: String(limit),
		offset: String(offset),
		order: 'clickcount',
		reverse: 'true',
		hidebroken: String(options.hideBroken ?? true)
	};
	const trimmed = query.trim();
	if (trimmed) params.name = trimmed;
	if (options.tag) params.tag = options.tag;
	if (options.country) params.country = options.country;
	if (options.countryCode) params.countrycode = options.countryCode;
	if (options.language) params.language = options.language;

	const stations = await rbFetch<RadioBrowserStation[]>('/stations/search', params);
	return {
		stations: stations.filter((s) => Boolean(s.url_resolved || s.url)).map(toDisplayStation),
		page,
		limit
	};
}

export async function getStation(uuid: string): Promise<DisplayRadioStation | null> {
	const list = await rbFetch<RadioBrowserStation[]>(`/stations/byuuid/${encodeURIComponent(uuid)}`);
	const first = list[0];
	return first ? toDisplayStation(first) : null;
}

export async function getTopStations(limit = 50): Promise<DisplayRadioStation[]> {
	const stations = await rbFetch<RadioBrowserStation[]>(
		`/stations/topclick/${Math.min(500, Math.max(1, limit))}`
	);
	return stations.filter((s) => Boolean(s.url_resolved || s.url)).map(toDisplayStation);
}

export async function getTags(limit = 200): Promise<RadioBrowserTag[]> {
	return rbFetch<RadioBrowserTag[]>('/tags', {
		order: 'stationcount',
		reverse: 'true',
		hidebroken: 'true',
		limit: String(Math.min(1000, Math.max(1, limit)))
	});
}

export async function getCountries(): Promise<RadioBrowserCountry[]> {
	return rbFetch<RadioBrowserCountry[]>('/countries', {
		order: 'stationcount',
		reverse: 'true',
		hidebroken: 'true'
	});
}

export async function getLanguages(): Promise<RadioBrowserLanguage[]> {
	return rbFetch<RadioBrowserLanguage[]>('/languages', {
		order: 'stationcount',
		reverse: 'true',
		hidebroken: 'true'
	});
}
