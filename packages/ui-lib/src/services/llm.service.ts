import { writable, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { fetchRaw } from 'ui-lib/transport/fetch-helpers';
import type {
	LlmState,
	LlmStatus,
	LocalModel,
	LlmConfigUpdate,
	LlmDownloadProgress
} from 'ui-lib/types/llm.type';

const initialState: LlmState = {
	status: null,
	models: [],
	downloadProgress: null,
	loading: false
};

class LlmService {
	public store: Writable<LlmState> = writable(initialState);
	private initialized = false;

	async initialize(): Promise<void> {
		if (!browser || this.initialized) return;
		this.initialized = true;

		await Promise.all([this.fetchStatus(), this.fetchModels()]);
	}

	async fetchStatus(): Promise<void> {
		try {
			const status = await this.fetchJson<LlmStatus>('/api/llm/status');
			this.store.update((s) => ({ ...s, status }));
		} catch (error) {
			console.error('[llm] Failed to fetch status:', error);
		}
	}

	async fetchModels(): Promise<void> {
		try {
			const models = await this.fetchJson<LocalModel[]>('/api/llm/models');
			this.store.update((s) => ({ ...s, models }));
		} catch (error) {
			console.error('[llm] Failed to fetch models:', error);
		}
	}

	async loadModel(fileName: string): Promise<void> {
		this.store.update((s) => ({ ...s, loading: true }));
		try {
			await this.fetchJson('/api/llm/models/load', {
				method: 'POST',
				body: JSON.stringify({ fileName })
			});
			await Promise.all([this.fetchStatus(), this.fetchModels()]);
		} catch (error) {
			console.error('[llm] Failed to load model:', error);
		} finally {
			this.store.update((s) => ({ ...s, loading: false }));
		}
	}

	async unloadModel(): Promise<void> {
		try {
			await this.fetchJson('/api/llm/models/unload', { method: 'POST' });
			await Promise.all([this.fetchStatus(), this.fetchModels()]);
		} catch (error) {
			console.error('[llm] Failed to unload model:', error);
		}
	}

	async downloadModel(repoId: string, fileName: string): Promise<void> {
		try {
			const response = await fetchRaw('/api/llm/models/download', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ repoId, fileName })
			});

			if (!response.ok || !response.body) {
				throw new Error(`Download request failed: ${response.status}`);
			}

			const reader = response.body.getReader();
			const decoder = new TextDecoder();
			let buffer = '';

			while (true) {
				const { done, value } = await reader.read();
				if (done) break;

				buffer += decoder.decode(value, { stream: true });
				const lines = buffer.split('\n');
				buffer = lines.pop() || '';

				for (const line of lines) {
					if (line.startsWith('data: ')) {
						try {
							const progress = JSON.parse(line.slice(6)) as LlmDownloadProgress;
							this.store.update((s) => ({ ...s, downloadProgress: progress }));

							if (progress.status === 'complete') {
								this.store.update((s) => ({ ...s, downloadProgress: null }));
								await this.fetchModels();
							}
						} catch {
							// ignore parse errors
						}
					}
				}
			}
		} catch (error) {
			console.error('[llm] Download failed:', error);
			this.store.update((s) => ({ ...s, downloadProgress: null }));
		}
	}

	async updateConfig(config: LlmConfigUpdate): Promise<void> {
		try {
			await this.fetchJson('/api/llm/config', {
				method: 'PUT',
				body: JSON.stringify(config)
			});
			await this.fetchStatus();
		} catch (error) {
			console.error('[llm] Failed to update config:', error);
		}
	}

	private async fetchJson<T>(path: string, init?: RequestInit): Promise<T> {
		const response = await fetchRaw(path, {
			...init,
			headers: {
				'Content-Type': 'application/json',
				...(init?.headers as Record<string, string>)
			}
		});

		if (!response.ok) {
			const body = await response.json().catch(() => ({}));
			throw new Error((body as { error?: string }).error || `Request failed: ${response.status}`);
		}

		const text = await response.text();
		return text ? JSON.parse(text) : ({} as T);
	}
}

export const llmService = new LlmService();
