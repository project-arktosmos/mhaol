import { writable, get } from 'svelte/store';
import { apiUrl } from 'frontend/lib/api-base';
import type {
	SmartSearchState,
	SmartSearchSelection,
	SmartSearchTorrentResult,
	TorrentAnalysis
} from 'frontend/types/smart-search.type';
import type { TorrentSearchResult } from 'addons/torrent-search-thepiratebay/types';

const initialState: SmartSearchState = {
	selection: null,
	visible: true,
	searching: false,
	analyzing: false,
	searchResults: [],
	searchError: null
};

class SmartSearchService {
	public store = writable(initialState);
	private abortController: AbortController | null = null;

	select(selection: SmartSearchSelection) {
		// Cancel any in-flight searches/analysis
		if (this.abortController) {
			this.abortController.abort();
		}
		this.abortController = new AbortController();

		this.store.update((s) => ({
			...s,
			selection,
			visible: true,
			searching: false,
			analyzing: false,
			searchResults: [],
			searchError: null
		}));
		this.runSearches(selection, this.abortController.signal);
	}

	clear() {
		if (this.abortController) {
			this.abortController.abort();
			this.abortController = null;
		}
		this.store.update((s) => ({
			...s,
			selection: null,
			searchResults: [],
			searchError: null,
			analyzing: false
		}));
	}

	private async runSearches(selection: SmartSearchSelection, signal: AbortSignal) {
		const { title, year } = selection;
		const queries = [
			title,
			`${title} ${year}`
		];

		this.store.update((s) => ({ ...s, searching: true, searchError: null }));

		try {
			const seen = new Map<string, SmartSearchTorrentResult>();
			const analyzeHashes = new Set<string>();

			for (const query of queries) {
				if (signal.aborted) return;

				try {
					const url = apiUrl(
						`/api/torrent/search?q=${encodeURIComponent(query)}&cat=200`
					);
					const res = await fetch(url, { signal });
					if (!res.ok) continue;
					const data: TorrentSearchResult[] = await res.json();

					// Pick top 5 from this query by SE then LE
					const sorted = [...data].sort((a, b) => {
						if (b.seeders !== a.seeders) return b.seeders - a.seeders;
						return b.leechers - a.leechers;
					});
					const top = sorted.slice(0, 5);
					for (const r of top) {
						analyzeHashes.add(r.infoHash);
					}

					for (const r of data) {
						const existing = seen.get(r.infoHash);
						if (existing) {
							existing.searchQueries.push(query);
						} else {
							seen.set(r.infoHash, {
								...r,
								uploadedAt: new Date(r.uploadedAt),
								searchQueries: [query],
								analysis: null,
								analyzing: false
							});
						}
					}
				} catch (e) {
					if (signal.aborted) return;
				}

				const current = [...seen.values()];
				this.store.update((s) => ({ ...s, searchResults: current }));
			}

			if (signal.aborted) return;
			this.store.update((s) => ({ ...s, searching: false }));

			await this.analyzeResults(selection, signal, analyzeHashes);
		} catch (error) {
			if (signal.aborted) return;
			this.store.update((s) => ({
				...s,
				searching: false,
				searchError: error instanceof Error ? error.message : String(error)
			}));
		}
	}

	private static BATCH_SIZE = 3;

	private async analyzeResults(selection: SmartSearchSelection, signal: AbortSignal, analyzeHashes: Set<string>) {
		const state = get(this.store);
		const toAnalyze = state.searchResults
			.map((r, i) => ({ result: r, index: i }))
			.filter((e) => analyzeHashes.has(e.result.infoHash));

		if (toAnalyze.length === 0) return;

		this.store.update((s) => ({ ...s, analyzing: true }));

		for (let start = 0; start < toAnalyze.length; start += SmartSearchService.BATCH_SIZE) {
			if (signal.aborted) return;

			const end = Math.min(start + SmartSearchService.BATCH_SIZE, toAnalyze.length);
			const batchEntries = toAnalyze.slice(start, end);
			const batch = batchEntries.map((e) => e.result);

			this.store.update((s) => {
				const results = [...s.searchResults];
				for (const entry of batchEntries) {
					results[entry.index] = { ...results[entry.index], analyzing: true };
				}
				return { ...s, searchResults: results };
			});

			const analyses = await this.callLlmBatch(batch, selection, signal);

			if (signal.aborted) return;

			this.store.update((s) => {
				const results = [...s.searchResults];
				for (let i = 0; i < batchEntries.length; i++) {
					results[batchEntries[i].index] = {
						...results[batchEntries[i].index],
						analysis: analyses[i] ?? null,
						analyzing: false
					};
				}
				return { ...s, searchResults: results };
			});
		}

		if (!signal.aborted) {
			this.store.update((s) => ({ ...s, analyzing: false }));
		}
	}

	private async callLlmBatch(
		batch: SmartSearchTorrentResult[],
		selection: SmartSearchSelection,
		signal: AbortSignal
	): Promise<(TorrentAnalysis | null)[]> {
		const listing = batch
			.map((r, i) => `${i + 1}. ${r.name}`)
			.join('\n');

		const prompt = `Target: "${selection.title}" (${selection.year})
${listing}
JSON array, one per torrent: [{"quality":"1080p","languages":"English","subs":"none","relevance":95,"reason":"matches title"}]`;

		try {
			const response = await fetch(apiUrl('/api/llm/chat/stream'), {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					messages: [
						{
							role: 'system',
							content: 'Extract from torrent filenames: quality, languages, subs, relevance 0-100 to target, reason. Return JSON array only.'
						},
						{ role: 'user', content: prompt }
					]
				}),
				signal
			});

			if (!response.ok || !response.body) return batch.map(() => null);

			const reader = response.body.getReader();
			const decoder = new TextDecoder();
			let buffer = '';
			let fullContent = '';

			while (true) {
				if (signal.aborted) {
					reader.cancel();
					return batch.map(() => null);
				}

				const { done, value } = await reader.read();
				if (done) break;

				buffer += decoder.decode(value, { stream: true });
				const lines = buffer.split('\n');
				buffer = lines.pop() || '';

				for (const line of lines) {
					if (line.startsWith('data: ')) {
						try {
							const event = JSON.parse(line.slice(6));
							if (event.done) break;
							fullContent += event.content;
						} catch {
							// ignore
						}
					}
				}
			}

			console.log('[smart-search] LLM raw response:', fullContent);

			// Strip markdown code fences if present
			const cleaned = fullContent.replace(/```json\s*/gi, '').replace(/```\s*/g, '');

			let parsed: Record<string, unknown>[] = [];

			// Try array first
			const arrayMatch = cleaned.match(/\[[\s\S]*\]/);
			if (arrayMatch) {
				try {
					parsed = JSON.parse(arrayMatch[0]);
				} catch {
					console.warn('[smart-search] Failed to parse JSON array');
				}
			}

			// Fallback: collect individual objects
			if (parsed.length === 0) {
				const objRegex = /\{[^{}]*\}/g;
				let m;
				while ((m = objRegex.exec(cleaned)) !== null) {
					try {
						parsed.push(JSON.parse(m[0]));
					} catch {
						// skip
					}
				}
			}

			console.log('[smart-search] Parsed results:', parsed);

			if (parsed.length === 0) {
				console.warn('[smart-search] No JSON found in response');
				return batch.map(() => null);
			}

			return batch.map((_, i) => {
				const p = parsed[i];
				if (!p) return null;
				const rel = p.relevance ?? p.score ?? p.match ?? 0;
				return {
					quality: String(p.quality ?? p.video_quality ?? 'Unknown'),
					languages: String(p.languages ?? p.language ?? p.lang ?? p.audio ?? 'Unknown'),
					subs: String(p.subs ?? p.subtitles ?? p.subtitle ?? 'none'),
					relevance: typeof rel === 'number' ? rel : parseInt(String(rel), 10) || 0,
					reason: String(p.reason ?? p.explanation ?? p.justification ?? '')
				};
			});
		} catch {
			return batch.map(() => null);
		}
	}
}

export const smartSearchService = new SmartSearchService();
