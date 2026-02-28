import type { PirateBayApiResult, TorrentCategory, TorrentSearchResult } from './types.js';
import { parseResults } from './parse.js';

const PIRATEBAY_API = 'https://apibay.org';
const DEFAULT_USER_AGENT = 'Mozilla/5.0 (compatible; Mhaol/1.0)';
const DEFAULT_TIMEOUT_MS = 30000;

export interface SearchOptions {
	category?: TorrentCategory;
	timeoutMs?: number;
	userAgent?: string;
}

export async function search(
	query: string,
	options: SearchOptions = {}
): Promise<TorrentSearchResult[]> {
	const trimmed = query.trim();
	if (!trimmed) return [];

	const category = options.category ?? '0';
	const timeoutMs = options.timeoutMs ?? DEFAULT_TIMEOUT_MS;
	const userAgent = options.userAgent ?? DEFAULT_USER_AGENT;

	const apiUrl = `${PIRATEBAY_API}/q.php?q=${encodeURIComponent(trimmed)}&cat=${encodeURIComponent(category)}`;

	const response = await fetch(apiUrl, {
		headers: { 'User-Agent': userAgent },
		signal: AbortSignal.timeout(timeoutMs)
	});

	if (!response.ok) {
		throw new Error(`PirateBay API returned ${response.status}`);
	}

	const rawResults: PirateBayApiResult[] = await response.json();
	return parseResults(rawResults);
}
