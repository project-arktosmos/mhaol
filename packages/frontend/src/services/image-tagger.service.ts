import { writable, get, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { apiUrl } from '$lib/api-base';
import type {
	ImageItem,
	ImageTag,
	ImagesResponse,
	TagResponse,
	BatchTagResponse,
	TaggerStatusResponse
} from '$types/image-tagger.type';

export type TaggerStatus = 'idle' | 'downloading' | 'loading' | 'ready' | 'error';

export interface ImageTaggerState {
	loading: boolean;
	error: string | null;
	taggerReady: boolean;
	taggerInitializing: boolean;
	taggerStatus: TaggerStatus;
	taggerProgress: number;
	taggerError: string | null;
	taggingItemIds: string[];
	filter: string;
}

const initialState: ImageTaggerState = {
	loading: false,
	error: null,
	taggerReady: false,
	taggerInitializing: false,
	taggerStatus: 'idle',
	taggerProgress: 0,
	taggerError: null,
	taggingItemIds: [],
	filter: ''
};

class ImageTaggerService {
	public store: Writable<ImageItem[]> = writable([]);
	public state: Writable<ImageTaggerState> = writable(initialState);

	private initialized = false;
	private pollTimer: ReturnType<typeof setInterval> | null = null;

	async initialize(): Promise<void> {
		if (!browser || this.initialized) return;

		this.state.update((s) => ({ ...s, loading: true, error: null }));

		try {
			const data = await this.fetchJson<ImagesResponse>('/api/images');
			this.store.set(data.images);
			this.initialized = true;
		} catch (error) {
			const errorMsg = error instanceof Error ? error.message : String(error);
			this.state.update((s) => ({ ...s, error: errorMsg }));
		} finally {
			this.state.update((s) => ({ ...s, loading: false }));
		}
	}

	async checkTaggerStatus(): Promise<void> {
		if (!browser) return;

		try {
			const data = await this.fetchJson<TaggerStatusResponse>('/api/images/tagger-status');
			this.state.update((s) => ({
				...s,
				taggerReady: data.ready,
				taggerStatus: data.status,
				taggerProgress: data.overallProgress,
				taggerError: data.error
			}));
		} catch {
			// Tagger status check failed silently
		}
	}

	private startProgressPolling(): void {
		if (this.pollTimer) return;
		this.pollTimer = setInterval(() => {
			this.checkTaggerStatus().then(() => {
				const { taggerStatus } = get(this.state);
				if (taggerStatus === 'ready' || taggerStatus === 'error') {
					this.stopProgressPolling();
				}
			});
		}, 500);
	}

	private stopProgressPolling(): void {
		if (this.pollTimer) {
			clearInterval(this.pollTimer);
			this.pollTimer = null;
		}
	}

	async tagImage(itemId: string): Promise<void> {
		if (!browser) return;

		this.state.update((s) => ({
			...s,
			taggerInitializing: true,
			taggingItemIds: [...s.taggingItemIds, itemId]
		}));
		this.startProgressPolling();

		try {
			const data = await this.fetchJson<TagResponse>('/api/images/tag', {
				method: 'POST',
				body: JSON.stringify({ libraryItemId: itemId })
			});

			this.store.update((items) =>
				items.map((item) => (item.id === itemId ? { ...item, tags: data.tags } : item))
			);

			this.state.update((s) => ({
				...s,
				taggerReady: true,
				taggerInitializing: false,
				taggerStatus: 'ready' as TaggerStatus
			}));
		} catch (error) {
			const errorMsg = error instanceof Error ? error.message : String(error);
			this.state.update((s) => ({
				...s,
				error: `Failed to tag image: ${errorMsg}`,
				taggerInitializing: false
			}));
		} finally {
			this.stopProgressPolling();
			this.state.update((s) => ({
				...s,
				taggingItemIds: s.taggingItemIds.filter((id) => id !== itemId)
			}));
		}
	}

	async tagBatch(itemIds: string[]): Promise<void> {
		if (!browser || itemIds.length === 0) return;

		this.state.update((s) => ({
			...s,
			taggerInitializing: true,
			taggingItemIds: [...s.taggingItemIds, ...itemIds]
		}));
		this.startProgressPolling();

		try {
			const data = await this.fetchJson<BatchTagResponse>('/api/images/tag-batch', {
				method: 'POST',
				body: JSON.stringify({ libraryItemIds: itemIds })
			});

			this.store.update((items) =>
				items.map((item) => {
					const tags = data.results[item.id];
					return tags ? { ...item, tags } : item;
				})
			);

			this.state.update((s) => ({
				...s,
				taggerReady: true,
				taggerInitializing: false,
				taggerStatus: 'ready' as TaggerStatus
			}));
		} catch (error) {
			const errorMsg = error instanceof Error ? error.message : String(error);
			this.state.update((s) => ({
				...s,
				error: `Failed to batch tag: ${errorMsg}`,
				taggerInitializing: false
			}));
		} finally {
			this.stopProgressPolling();
			this.state.update((s) => ({
				...s,
				taggingItemIds: s.taggingItemIds.filter((id) => !itemIds.includes(id))
			}));
		}
	}

	async addTag(itemId: string, tag: string): Promise<void> {
		if (!browser) return;

		await this.fetchJson('/api/images/tags', {
			method: 'POST',
			body: JSON.stringify({ libraryItemId: itemId, tag })
		});

		this.store.update((items) =>
			items.map((item) =>
				item.id === itemId
					? { ...item, tags: [...item.tags, { tag: tag.trim().toLowerCase(), score: 1.0 }] }
					: item
			)
		);
	}

	async removeTag(itemId: string, tag: string): Promise<void> {
		if (!browser) return;

		await this.fetchJson('/api/images/tags', {
			method: 'DELETE',
			body: JSON.stringify({ libraryItemId: itemId, tag })
		});

		this.store.update((items) =>
			items.map((item) =>
				item.id === itemId
					? { ...item, tags: item.tags.filter((t) => t.tag !== tag) }
					: item
			)
		);
	}

	setFilter(value: string): void {
		this.state.update((s) => ({ ...s, filter: value }));
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

export const imageTaggerService = new ImageTaggerService();
