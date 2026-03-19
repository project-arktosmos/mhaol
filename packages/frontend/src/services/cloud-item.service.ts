import { writable, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { apiUrl } from 'frontend/lib/api-base';
import type { CloudItem } from 'frontend/types/cloud.type';

export interface CloudItemServiceState {
	currentItem: CloudItem | null;
	loading: boolean;
	searchResults: CloudItem[];
	searchLoading: boolean;
	distinctKeys: string[];
	distinctValues: Record<string, string[]>;
}

const initialState: CloudItemServiceState = {
	currentItem: null,
	loading: false,
	searchResults: [],
	searchLoading: false,
	distinctKeys: [],
	distinctValues: {}
};

class CloudItemService {
	public state: Writable<CloudItemServiceState> = writable(initialState);

	async getItem(itemId: string): Promise<CloudItem | null> {
		if (!browser) return null;

		this.state.update((s) => ({ ...s, loading: true }));

		try {
			const item = await this.fetchJson<CloudItem>(`/api/cloud/items/${itemId}`);
			this.state.update((s) => ({ ...s, currentItem: item, loading: false }));
			return item;
		} catch (error) {
			console.error('[cloud-item] Failed to get item:', error);
			this.state.update((s) => ({ ...s, loading: false }));
			return null;
		}
	}

	async setAttribute(
		itemId: string,
		key: string,
		value: string,
		typeId: string = 'string'
	): Promise<void> {
		if (!browser) return;

		await this.fetchJson(`/api/cloud/items/${itemId}/attributes`, {
			method: 'PUT',
			body: JSON.stringify([{ key, value, typeId, source: 'user' }])
		});

		this.state.update((s) => {
			if (!s.currentItem || s.currentItem.id !== itemId) return s;
			const attrs = s.currentItem.attributes.filter((a) => !(a.key === key && a.source === 'user'));
			attrs.push({ key, value, typeId, source: 'user', confidence: null });
			return { ...s, currentItem: { ...s.currentItem, attributes: attrs } };
		});
	}

	async removeAttribute(itemId: string, key: string): Promise<void> {
		if (!browser) return;

		await this.fetchJson(`/api/cloud/items/${itemId}/attributes/${encodeURIComponent(key)}`, {
			method: 'DELETE'
		});

		this.state.update((s) => {
			if (!s.currentItem || s.currentItem.id !== itemId) return s;
			return {
				...s,
				currentItem: {
					...s.currentItem,
					attributes: s.currentItem.attributes.filter((a) => a.key !== key)
				}
			};
		});
	}

	async search(query?: string, key?: string, value?: string): Promise<void> {
		if (!browser) return;

		this.state.update((s) => ({ ...s, searchLoading: true }));

		try {
			const params = new URLSearchParams();
			if (query) params.set('q', query);
			if (key) params.set('key', key);
			if (value) params.set('value', value);

			const results = await this.fetchJson<CloudItem[]>(`/api/cloud/search?${params.toString()}`);
			this.state.update((s) => ({ ...s, searchResults: results, searchLoading: false }));
		} catch (error) {
			console.error('[cloud-item] Failed to search:', error);
			this.state.update((s) => ({ ...s, searchLoading: false }));
		}
	}

	async fetchDistinctKeys(): Promise<void> {
		if (!browser) return;

		try {
			const keys = await this.fetchJson<string[]>('/api/cloud/attributes/keys');
			this.state.update((s) => ({ ...s, distinctKeys: keys }));
		} catch (error) {
			console.error('[cloud-item] Failed to fetch distinct keys:', error);
		}
	}

	async fetchDistinctValues(key: string): Promise<void> {
		if (!browser) return;

		try {
			const values = await this.fetchJson<string[]>(
				`/api/cloud/attributes/values/${encodeURIComponent(key)}`
			);
			this.state.update((s) => ({
				...s,
				distinctValues: { ...s.distinctValues, [key]: values }
			}));
		} catch (error) {
			console.error('[cloud-item] Failed to fetch distinct values:', error);
		}
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

export const cloudItemService = new CloudItemService();
