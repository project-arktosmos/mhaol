import { writable, get, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { apiUrl } from 'frontend/lib/api-base';
import type { Lyrics, LyricsState } from 'lyrics/types';
import type { PlayableFile } from 'frontend/types/player.type';

class LyricsService {
	store: Writable<LyricsState>;
	private cache: Map<string, Lyrics> = new Map();

	constructor() {
		this.store = writable<LyricsState>({
			status: 'idle',
			lyrics: null,
			error: null,
			currentTrackId: null
		});
	}

	async fetchForItemId(itemId: string): Promise<void> {
		if (!browser) return;
		return this.fetchById(itemId);
	}

	async fetchForFile(file: PlayableFile): Promise<void> {
		if (!browser) return;

		if (file.type !== 'library') {
			this.clear();
			return;
		}

		return this.fetchById(file.id);
	}

	private async fetchById(cacheKey: string): Promise<void> {
		const cached = this.cache.get(cacheKey);
		if (cached) {
			this.store.set({
				status: cached.plainLyrics || cached.syncedLyrics ? 'success' : 'not_found',
				lyrics: cached,
				error: null,
				currentTrackId: cacheKey
			});
			return;
		}

		this.store.update((s) => ({
			...s,
			status: 'loading',
			error: null,
			currentTrackId: cacheKey
		}));

		try {
			const response = await fetch(apiUrl(`/api/lyrics/${cacheKey}`));

			if (!response.ok) {
				const status = response.status === 404 ? 'not_found' : 'error';
				this.store.set({
					status,
					lyrics: null,
					error: status === 'error' ? `HTTP ${response.status}` : null,
					currentTrackId: cacheKey
				});
				return;
			}

			const lyrics = (await response.json()) as Lyrics;
			this.cache.set(cacheKey, lyrics);

			const hasContent = lyrics.plainLyrics || lyrics.syncedLyrics || lyrics.instrumental;
			this.store.set({
				status: hasContent ? 'success' : 'not_found',
				lyrics,
				error: null,
				currentTrackId: cacheKey
			});
		} catch (error) {
			this.store.set({
				status: 'error',
				lyrics: null,
				error: error instanceof Error ? error.message : 'Failed to fetch lyrics',
				currentTrackId: cacheKey
			});
		}
	}

	getCurrentLineIndex(currentTime: number): number {
		const state = get(this.store);
		if (!state.lyrics?.syncedLyrics) return -1;

		const lines = state.lyrics.syncedLyrics;
		let currentIndex = -1;

		for (let i = 0; i < lines.length; i++) {
			if (lines[i].time <= currentTime) {
				currentIndex = i;
			} else {
				break;
			}
		}

		return currentIndex;
	}

	clear(): void {
		this.store.set({
			status: 'idle',
			lyrics: null,
			error: null,
			currentTrackId: null
		});
	}
}

export const lyricsService = new LyricsService();
