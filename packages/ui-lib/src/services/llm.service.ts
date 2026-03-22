import { writable, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { apiUrl } from 'ui-lib/lib/api-base';
import type {
	LlmState,
	LlmStatus,
	LocalModel,
	LlmConversation,
	ChatMessage,
	LlmConfigUpdate,
	LlmDownloadProgress,
	LlmTokenEvent
} from 'ui-lib/types/llm.type';

const initialState: LlmState = {
	status: null,
	models: [],
	conversations: [],
	activeConversationId: null,
	messages: [],
	streamingContent: '',
	isGenerating: false,
	downloadProgress: null,
	loading: false
};

class LlmService {
	public store: Writable<LlmState> = writable(initialState);
	private initialized = false;
	private abortController: AbortController | null = null;

	async initialize(): Promise<void> {
		if (!browser || this.initialized) return;
		this.initialized = true;

		await Promise.all([this.fetchStatus(), this.fetchModels(), this.fetchConversations()]);
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
			const response = await fetch(apiUrl('/api/llm/models/download'), {
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

	async sendMessage(content: string): Promise<void> {
		this.store.update((s) => {
			const newMessages: ChatMessage[] = [...s.messages, { role: 'user', content }];
			return { ...s, messages: newMessages, streamingContent: '', isGenerating: true };
		});

		try {
			let currentState: LlmState;
			this.store.subscribe((s) => (currentState = s))();

			this.abortController = new AbortController();

			const response = await fetch(apiUrl('/api/llm/chat/stream'), {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ messages: currentState!.messages }),
				signal: this.abortController.signal
			});

			if (!response.ok || !response.body) {
				throw new Error(`Chat request failed: ${response.status}`);
			}

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
							const event = JSON.parse(line.slice(6)) as LlmTokenEvent;
							if (event.done) {
								this.store.update((s) => ({
									...s,
									messages: [...s.messages, { role: 'assistant', content: fullContent }],
									streamingContent: '',
									isGenerating: false
								}));
								this.saveActiveConversation();
								return;
							}
							fullContent += event.content;
							this.store.update((s) => ({ ...s, streamingContent: fullContent }));
						} catch {
							// ignore parse errors
						}
					}
				}
			}

			// Stream ended without done event
			if (fullContent) {
				this.store.update((s) => ({
					...s,
					messages: [...s.messages, { role: 'assistant', content: fullContent }],
					streamingContent: '',
					isGenerating: false
				}));
				this.saveActiveConversation();
			}
		} catch (error) {
			if ((error as Error).name !== 'AbortError') {
				console.error('[llm] Chat stream failed:', error);
			}
			this.store.update((s) => ({ ...s, streamingContent: '', isGenerating: false }));
		} finally {
			this.abortController = null;
		}
	}

	async cancelGeneration(): Promise<void> {
		this.abortController?.abort();
		try {
			await this.fetchJson('/api/llm/chat/cancel', { method: 'POST' });
		} catch {
			// best effort
		}
		this.store.update((s) => ({ ...s, isGenerating: false, streamingContent: '' }));
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

	// -- Conversations --

	async fetchConversations(): Promise<void> {
		try {
			const conversations = await this.fetchJson<LlmConversation[]>('/api/llm/conversations');
			this.store.update((s) => ({ ...s, conversations }));
		} catch (error) {
			console.error('[llm] Failed to fetch conversations:', error);
		}
	}

	async createConversation(title: string, systemPrompt?: string): Promise<void> {
		try {
			const conversation = await this.fetchJson<LlmConversation>('/api/llm/conversations', {
				method: 'POST',
				body: JSON.stringify({ title, systemPrompt: systemPrompt || null })
			});
			this.store.update((s) => ({
				...s,
				conversations: [conversation, ...s.conversations],
				activeConversationId: conversation.id,
				messages: []
			}));
		} catch (error) {
			console.error('[llm] Failed to create conversation:', error);
		}
	}

	async selectConversation(id: string): Promise<void> {
		try {
			const conversation = await this.fetchJson<LlmConversation>(`/api/llm/conversations/${id}`);
			const messages: ChatMessage[] = JSON.parse(conversation.messages || '[]');
			this.store.update((s) => ({
				...s,
				activeConversationId: id,
				messages,
				streamingContent: ''
			}));
		} catch (error) {
			console.error('[llm] Failed to select conversation:', error);
		}
	}

	async deleteConversation(id: string): Promise<void> {
		try {
			await this.fetchJson(`/api/llm/conversations/${id}`, { method: 'DELETE' });
			this.store.update((s) => {
				const conversations = s.conversations.filter((c) => c.id !== id);
				const isActive = s.activeConversationId === id;
				return {
					...s,
					conversations,
					activeConversationId: isActive ? null : s.activeConversationId,
					messages: isActive ? [] : s.messages
				};
			});
		} catch (error) {
			console.error('[llm] Failed to delete conversation:', error);
		}
	}

	private async saveActiveConversation(): Promise<void> {
		let state: LlmState;
		this.store.subscribe((s) => (state = s))();

		if (!state!.activeConversationId) return;

		const conversation = state!.conversations.find((c) => c.id === state!.activeConversationId);
		if (!conversation) return;

		try {
			await this.fetchJson(`/api/llm/conversations/${conversation.id}`, {
				method: 'PUT',
				body: JSON.stringify({
					title: conversation.title,
					messages: JSON.stringify(state!.messages)
				})
			});
		} catch (error) {
			console.error('[llm] Failed to save conversation:', error);
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
			throw new Error(body.error || `Request failed: ${response.status}`);
		}

		const text = await response.text();
		return text ? JSON.parse(text) : ({} as T);
	}
}

export const llmService = new LlmService();
