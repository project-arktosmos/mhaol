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

	select(selection: SmartSearchSelection) {
		this.store.update((s) => ({
			...s,
			selection,
			visible: true,
			searching: false,
			analyzing: false,
			searchResults: [],
			searchError: null
		}));
		this.runSearches(selection);
	}

	clear() {
		this.store.update((s) => ({
			...s,
			selection: null,
			searchResults: [],
			searchError: null,
			analyzing: false
		}));
	}

	private async runSearches(selection: SmartSearchSelection) {
		const { title, year, type } = selection;
		const typeLabel = type === 'movie' ? 'movie' : 'tv show';
		const queries = [
			title,
			`${title} ${year}`,
			`${title} ${typeLabel}`,
			`${title} ${year} ${typeLabel}`
		];

		this.store.update((s) => ({ ...s, searching: true, searchError: null }));

		try {
			const seen = new Map<string, SmartSearchTorrentResult>();

			for (const query of queries) {
				try {
					const url = apiUrl(
						`/api/torrent/search?q=${encodeURIComponent(query)}&cat=200`
					);
					const res = await fetch(url);
					if (!res.ok) continue;
					const data: TorrentSearchResult[] = await res.json();

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
				} catch {
					// skip failed query
				}

				const current = [...seen.values()].sort((a, b) => b.seeders - a.seeders);
				this.store.update((s) => ({ ...s, searchResults: current }));
			}

			this.store.update((s) => ({ ...s, searching: false }));

			// Start LLM analysis
			this.analyzeResults(selection);
		} catch (error) {
			this.store.update((s) => ({
				...s,
				searching: false,
				searchError: error instanceof Error ? error.message : String(error)
			}));
		}
	}

	private async analyzeResults(selection: SmartSearchSelection) {
		const state = get(this.store);
		if (state.searchResults.length === 0) return;

		this.store.update((s) => ({ ...s, analyzing: true }));

		for (let i = 0; i < state.searchResults.length; i++) {
			const result = state.searchResults[i];

			// Mark this result as analyzing
			this.store.update((s) => {
				const results = [...s.searchResults];
				results[i] = { ...results[i], analyzing: true };
				return { ...s, searchResults: results };
			});

			const analysis = await this.analyzeOneResult(result, selection);

			// Store the analysis
			this.store.update((s) => {
				const results = [...s.searchResults];
				results[i] = { ...results[i], analysis, analyzing: false };
				return { ...s, searchResults: results };
			});
		}

		this.store.update((s) => ({ ...s, analyzing: false }));
	}

	private async analyzeOneResult(
		result: SmartSearchTorrentResult,
		selection: SmartSearchSelection
	): Promise<TorrentAnalysis | null> {
		const prompt = `Analyze this torrent result for the ${selection.type === 'movie' ? 'movie' : 'TV show'} "${selection.title}" (${selection.year}).

Torrent name: "${result.name}"
Matched search queries: ${result.searchQueries.join(', ')}

Respond ONLY with a JSON object, no other text:
{"quality":"video quality (e.g. 1080p, 720p, 4K, CAM, TS, WEBSCREENER)","languages":"audio language(s)","subs":"subtitle language(s) or none","relevance":0-100,"reason":"one sentence justifying the relevance percentage"}`;

		try {
			const response = await fetch(apiUrl('/api/llm/chat/stream'), {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					messages: [
						{ role: 'system', content: 'You analyze torrent filenames. Respond only with valid JSON, no markdown, no explanation.' },
						{ role: 'user', content: prompt }
					]
				})
			});

			if (!response.ok || !response.body) return null;

			const reader = response.body.getReader();
			const decoder = new TextDecoder();
			let buffer = '';
			let fullContent = '';

			while (true) {
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

			// Extract JSON from response
			const jsonMatch = fullContent.match(/\{[\s\S]*\}/);
			if (!jsonMatch) return null;

			const parsed = JSON.parse(jsonMatch[0]);
			return {
				quality: parsed.quality ?? 'Unknown',
				languages: parsed.languages ?? 'Unknown',
				subs: parsed.subs ?? 'none',
				relevance: typeof parsed.relevance === 'number' ? parsed.relevance : 0,
				reason: parsed.reason ?? ''
			};
		} catch {
			return null;
		}
	}
}

export const smartSearchService = new SmartSearchService();
