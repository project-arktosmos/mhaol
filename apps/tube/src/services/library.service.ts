import { writable, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { apiUrl } from '$lib/api-base';
import type { Library } from '$types/library.type';
import type { YouTubeContent } from '$types/youtube.type';

export interface LibraryServiceState {
	content: YouTubeContent[];
	contentLoading: boolean;
	contentError: string | null;
	favorites: YouTubeContent[];
	favoritesLoading: boolean;
}

const initialState: LibraryServiceState = {
	content: [],
	contentLoading: false,
	contentError: null,
	favorites: [],
	favoritesLoading: false
};

class LibraryService {
	public library: Writable<Library | null> = writable(null);
	public state: Writable<LibraryServiceState> = writable(initialState);

	private initialized = false;

	reset(): void {
		this.initialized = false;
		this.library.set(null);
		this.state.set(initialState);
	}

	async initialize(): Promise<void> {
		if (!browser || this.initialized) return;

		try {
			const library = await this.fetchJson<Library>('/api/libraries');
			this.library.set(library);
			this.initialized = true;
			this.fetchContent();
			this.fetchFavorites();
		} catch (error) {
			console.error('[library] Failed to initialize:', error);
		}
	}

	async fetchContent(): Promise<void> {
		if (!browser) return;

		this.state.update((s) => ({ ...s, contentLoading: true, contentError: null }));

		try {
			const content = await this.fetchJson<YouTubeContent[]>('/api/media');
			this.state.update((s) => ({ ...s, content, contentLoading: false }));

			if (content.some((c) => c.durationSeconds == null)) {
				this.fillMissingDurations();
			}
		} catch (error) {
			const errorMsg = error instanceof Error ? error.message : String(error);
			this.state.update((s) => ({ ...s, contentLoading: false, contentError: errorMsg }));
		}
	}

	private async fillMissingDurations(): Promise<void> {
		try {
			const filled = await this.fetchJson<{ youtubeId: string; durationSeconds: number }[]>(
				'/api/media/fill-durations',
				{ method: 'POST' }
			);
			if (filled.length === 0) return;
			const map = new Map(filled.map((f) => [f.youtubeId, f.durationSeconds]));
			this.state.update((s) => ({
				...s,
				content: s.content.map((c) =>
					map.has(c.youtubeId) ? { ...c, durationSeconds: map.get(c.youtubeId)! } : c
				)
			}));
		} catch {
			// non-critical
		}
	}

	async fetchFavorites(): Promise<void> {
		if (!browser) return;

		this.state.update((s) => ({ ...s, favoritesLoading: true }));

		try {
			const favorites = await this.fetchJson<YouTubeContent[]>('/api/media/favorites');
			this.state.update((s) => ({ ...s, favorites, favoritesLoading: false }));
		} catch {
			this.state.update((s) => ({ ...s, favoritesLoading: false }));
		}
	}

	streamVideoUrl(youtubeId: string): string {
		return apiUrl(`/api/libraries/content/${youtubeId}/stream/video`);
	}

	streamAudioUrl(youtubeId: string): string {
		return apiUrl(`/api/libraries/content/${youtubeId}/stream/audio`);
	}

	streamDownloadVideoUrl(downloadId: string): string {
		return apiUrl(`/api/ytdl/downloads/${downloadId}/stream/video`);
	}

	async toggleFavorite(youtubeId: string): Promise<boolean> {
		const result = await this.fetchJson<{ isFavorite: boolean }>(
			`/api/media/${youtubeId}/favorite`,
			{
				method: 'PUT'
			}
		);
		this.state.update((s) => ({
			...s,
			content: s.content.map((c) =>
				c.youtubeId === youtubeId ? { ...c, isFavorite: result.isFavorite } : c
			)
		}));
		this.fetchFavorites();
		return result.isFavorite;
	}

	async deleteAudio(youtubeId: string): Promise<void> {
		const response = await fetch(apiUrl(`/api/media/${youtubeId}/audio`), { method: 'DELETE' });
		if (!response.ok) throw new Error(`HTTP ${response.status}`);
		await this.fetchContent();
	}

	async deleteVideo(youtubeId: string): Promise<void> {
		const response = await fetch(apiUrl(`/api/media/${youtubeId}/video`), { method: 'DELETE' });
		if (!response.ok) throw new Error(`HTTP ${response.status}`);
		await this.fetchContent();
	}

	private async fetchJson<T>(path: string, init?: RequestInit): Promise<T> {
		const response = await fetch(apiUrl(path), {
			...init,
			headers: {
				'Content-Type': 'application/json',
				...init?.headers
			}
		});

		if (!response.ok) {
			const body = await response.json().catch(() => ({}));
			throw new Error((body as { error?: string }).error ?? `HTTP ${response.status}`);
		}

		return response.json() as Promise<T>;
	}
}

export const libraryService = new LibraryService();
