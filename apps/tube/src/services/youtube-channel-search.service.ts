import { writable, get, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { apiUrl } from '$lib/api-base';
import type { YouTubeSearchChannelItem, YouTubeSearchResponse } from '$types/youtube-search.type';

interface YouTubeChannelSearchState {
	query: string;
	searching: boolean;
	channels: YouTubeSearchChannelItem[];
	continuation: string | null;
	loadingMore: boolean;
	error: string | null;
}

const initialState: YouTubeChannelSearchState = {
	query: '',
	searching: false,
	channels: [],
	continuation: null,
	loadingMore: false,
	error: null
};

class YouTubeChannelSearchService {
	public state: Writable<YouTubeChannelSearchState> = writable(initialState);

	async search(query: string): Promise<void> {
		if (!browser || !query.trim()) return;

		this.state.update((s) => ({
			...s,
			query: query.trim(),
			searching: true,
			error: null,
			channels: [],
			continuation: null
		}));

		try {
			const params = new URLSearchParams({ q: query.trim() });
			const response = await fetch(apiUrl(`/api/youtube-search/search?${params}`));

			if (!response.ok) {
				const body = await response.json().catch(() => ({}));
				throw new Error((body as { error?: string }).error ?? `HTTP ${response.status}`);
			}

			const data: YouTubeSearchResponse = await response.json();

			this.state.update((s) => ({
				...s,
				searching: false,
				channels: data.channels,
				continuation: data.continuation
			}));
		} catch (error) {
			const msg = error instanceof Error ? error.message : String(error);
			this.state.update((s) => ({
				...s,
				searching: false,
				error: `Search failed: ${msg}`
			}));
		}
	}

	async loadMore(): Promise<void> {
		if (!browser) return;

		const current = get(this.state);
		if (!current.continuation || current.loadingMore) return;

		this.state.update((s) => ({ ...s, loadingMore: true }));

		try {
			const params = new URLSearchParams({
				q: current.query,
				continuation: current.continuation!
			});
			const response = await fetch(apiUrl(`/api/youtube-search/search?${params}`));

			if (!response.ok) {
				const body = await response.json().catch(() => ({}));
				throw new Error((body as { error?: string }).error ?? `HTTP ${response.status}`);
			}

			const data: YouTubeSearchResponse = await response.json();

			this.state.update((s) => ({
				...s,
				loadingMore: false,
				channels: [...s.channels, ...data.channels],
				continuation: data.continuation
			}));
		} catch (error) {
			const msg = error instanceof Error ? error.message : String(error);
			this.state.update((s) => ({
				...s,
				loadingMore: false,
				error: `Load more failed: ${msg}`
			}));
		}
	}

	clearResults(): void {
		this.state.set(initialState);
	}
}

export const youtubeChannelSearchService = new YouTubeChannelSearchService();
