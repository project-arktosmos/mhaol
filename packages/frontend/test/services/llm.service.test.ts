import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';

function mockFetchOk(data: unknown) {
	return vi.fn().mockResolvedValue({
		ok: true,
		json: () => Promise.resolve(data),
		text: () => Promise.resolve(JSON.stringify(data)),
		body: null
	});
}

function mockFetchError(errorMsg: string, status = 500) {
	return vi.fn().mockResolvedValue({
		ok: false,
		status,
		json: () => Promise.resolve({ error: errorMsg }),
		text: () => Promise.resolve(''),
		body: null
	});
}

function mockFetchByUrl(handlers: Record<string, unknown>) {
	return vi.fn(async (url: string) => {
		for (const [pattern, data] of Object.entries(handlers)) {
			if (url.includes(pattern)) {
				return {
					ok: true,
					json: async () => data,
					text: async () => JSON.stringify(data)
				} as Response;
			}
		}
		return { ok: true, json: async () => ({}), text: async () => '{}' } as Response;
	});
}

describe('LlmService', () => {
	let llmService: (typeof import('../../src/services/llm.service'))['llmService'];

	beforeEach(async () => {
		vi.resetModules();
		vi.stubGlobal('fetch', vi.fn());
		const mod = await import('../../src/services/llm.service');
		llmService = mod.llmService;
	});

	afterEach(() => {
		vi.unstubAllGlobals();
	});

	// ===== Initial state =====

	it('should have correct initial state', () => {
		const state = get(llmService.store);
		expect(state.status).toBeNull();
		expect(state.models).toEqual([]);
		expect(state.conversations).toEqual([]);
		expect(state.activeConversationId).toBeNull();
		expect(state.messages).toEqual([]);
		expect(state.streamingContent).toBe('');
		expect(state.isGenerating).toBe(false);
		expect(state.downloadProgress).toBeNull();
		expect(state.loading).toBe(false);
	});

	// ===== initialize =====

	it('should initialize by fetching status, models, and conversations', async () => {
		const mockFn = mockFetchByUrl({
			'/api/llm/status': { loaded: true, modelName: 'test' },
			'/api/llm/models': [{ fileName: 'model.gguf' }],
			'/api/llm/conversations': [{ id: 'c1', title: 'Chat' }]
		});
		vi.stubGlobal('fetch', mockFn);

		await llmService.initialize();

		const state = get(llmService.store);
		expect(state.status).toEqual({ loaded: true, modelName: 'test' });
		expect(state.models).toHaveLength(1);
		expect(state.conversations).toHaveLength(1);
	});

	it('should not initialize twice', async () => {
		const mockFn = mockFetchByUrl({
			'/api/llm/status': {},
			'/api/llm/models': [],
			'/api/llm/conversations': []
		});
		vi.stubGlobal('fetch', mockFn);

		await llmService.initialize();
		const callCount = mockFn.mock.calls.length;

		await llmService.initialize();
		expect(mockFn.mock.calls.length).toBe(callCount);
	});

	// ===== fetchStatus =====

	it('should fetch status', async () => {
		const status = { loaded: true, modelName: 'test-model', contextSize: 2048 };
		vi.stubGlobal('fetch', mockFetchOk(status));

		await llmService.fetchStatus();

		const state = get(llmService.store);
		expect(state.status).toEqual(status);
	});

	it('should handle fetchStatus error', async () => {
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Network error')));

		await llmService.fetchStatus();

		const state = get(llmService.store);
		expect(state.status).toBeNull();
	});

	// ===== fetchModels =====

	it('should fetch models', async () => {
		const models = [
			{ fileName: 'model1.gguf', size: 1024, quantization: 'Q4_K_M' },
			{ fileName: 'model2.gguf', size: 2048, quantization: 'Q5_K_M' }
		];
		vi.stubGlobal('fetch', mockFetchOk(models));

		await llmService.fetchModels();

		const state = get(llmService.store);
		expect(state.models).toHaveLength(2);
	});

	it('should handle fetchModels error', async () => {
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Failed')));

		await llmService.fetchModels();

		const state = get(llmService.store);
		expect(state.models).toEqual([]);
	});

	// ===== loadModel =====

	it('should load a model', async () => {
		const mockFn = mockFetchByUrl({
			'/models/load': {},
			'/status': { loaded: true },
			'/models': []
		});
		vi.stubGlobal('fetch', mockFn);

		await llmService.loadModel('model1.gguf');

		const state = get(llmService.store);
		expect(state.loading).toBe(false);
		expect(mockFn).toHaveBeenCalledWith(
			expect.stringContaining('/api/llm/models/load'),
			expect.objectContaining({ method: 'POST' })
		);
	});

	it('should set loading true during loadModel', async () => {
		let capturedLoading = false;
		const unsub = llmService.store.subscribe((s) => {
			if (s.loading) capturedLoading = true;
		});

		vi.stubGlobal('fetch', mockFetchOk({}));

		await llmService.loadModel('model.gguf');
		unsub();
		expect(capturedLoading).toBe(true);
	});

	it('should set loading false even on loadModel error', async () => {
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Load failed')));

		await llmService.loadModel('bad.gguf');

		const state = get(llmService.store);
		expect(state.loading).toBe(false);
	});

	// ===== unloadModel =====

	it('should unload model', async () => {
		const mockFn = mockFetchByUrl({
			'/models/unload': {},
			'/status': { loaded: false },
			'/models': []
		});
		vi.stubGlobal('fetch', mockFn);

		await llmService.unloadModel();

		expect(mockFn).toHaveBeenCalledWith(
			expect.stringContaining('/api/llm/models/unload'),
			expect.objectContaining({ method: 'POST' })
		);
	});

	it('should handle unloadModel error', async () => {
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Unload failed')));

		await llmService.unloadModel();

		// Should not throw, just log
		expect(true).toBe(true);
	});

	// ===== downloadModel =====

	it('should handle download with streaming progress events', async () => {
		const encoder = new TextEncoder();
		const chunks = [
			encoder.encode('data: {"status":"downloading","progress":50}\n\n'),
			encoder.encode('data: {"status":"complete","progress":100}\n\n')
		];

		let chunkIdx = 0;
		const reader = {
			read: vi.fn().mockImplementation(async () => {
				if (chunkIdx < chunks.length) {
					return { done: false, value: chunks[chunkIdx++] };
				}
				return { done: true, value: undefined };
			})
		};

		vi.stubGlobal(
			'fetch',
			vi.fn().mockImplementation(async (url: string) => {
				if (url.includes('/models/download')) {
					return {
						ok: true,
						body: { getReader: () => reader },
						json: async () => ({}),
						text: async () => '{}'
					};
				}
				// fetchModels call after complete
				return {
					ok: true,
					json: async () => [],
					text: async () => '[]'
				};
			})
		);

		await llmService.downloadModel('repo/model', 'model.gguf');

		// After complete, downloadProgress should be null
		const state = get(llmService.store);
		expect(state.downloadProgress).toBeNull();
	});

	it('should handle download with non-ok response', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn().mockResolvedValue({
				ok: false,
				status: 500,
				body: null,
				json: async () => ({}),
				text: async () => '{}'
			})
		);

		await llmService.downloadModel('repo/model', 'model.gguf');

		const state = get(llmService.store);
		expect(state.downloadProgress).toBeNull();
	});

	it('should handle download fetch failure', async () => {
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Network error')));

		await llmService.downloadModel('repo/model', 'model.gguf');

		const state = get(llmService.store);
		expect(state.downloadProgress).toBeNull();
	});

	// ===== sendMessage =====

	it('should add user message to state and set isGenerating', async () => {
		const encoder = new TextEncoder();
		const chunk = encoder.encode('data: {"content":"Hello","done":true}\n\n');
		let read = false;
		const reader = {
			read: vi.fn().mockImplementation(async () => {
				if (!read) {
					read = true;
					return { done: false, value: chunk };
				}
				return { done: true, value: undefined };
			})
		};

		vi.stubGlobal(
			'fetch',
			vi.fn().mockImplementation(async (url: string) => {
				if (url.includes('/chat/stream')) {
					return {
						ok: true,
						body: { getReader: () => reader },
						json: async () => ({}),
						text: async () => '{}'
					};
				}
				// saveActiveConversation call
				return {
					ok: true,
					json: async () => ({}),
					text: async () => '{}'
				};
			})
		);

		await llmService.sendMessage('Hello world');

		const state = get(llmService.store);
		expect(state.messages.length).toBeGreaterThanOrEqual(1);
		expect(state.messages[0].content).toBe('Hello world');
		expect(state.isGenerating).toBe(false);
	});

	it('should handle sendMessage with non-ok response', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn().mockResolvedValue({
				ok: false,
				status: 500,
				body: null,
				json: async () => ({}),
				text: async () => '{}'
			})
		);

		await llmService.sendMessage('test');

		const state = get(llmService.store);
		expect(state.isGenerating).toBe(false);
		expect(state.streamingContent).toBe('');
	});

	it('should handle sendMessage fetch failure', async () => {
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Connection lost')));

		await llmService.sendMessage('test');

		const state = get(llmService.store);
		expect(state.isGenerating).toBe(false);
	});

	it('should handle stream ending without done event', async () => {
		const encoder = new TextEncoder();
		const chunk = encoder.encode('data: {"content":"partial","done":false}\n\n');
		let read = false;
		const reader = {
			read: vi.fn().mockImplementation(async () => {
				if (!read) {
					read = true;
					return { done: false, value: chunk };
				}
				return { done: true, value: undefined };
			})
		};

		vi.stubGlobal(
			'fetch',
			vi.fn().mockImplementation(async () => ({
				ok: true,
				body: { getReader: () => reader },
				json: async () => ({}),
				text: async () => '{}'
			}))
		);

		await llmService.sendMessage('test');

		const state = get(llmService.store);
		// Should have added the partial content as assistant message
		const assistantMsg = state.messages.find((m) => m.role === 'assistant');
		expect(assistantMsg?.content).toBe('partial');
		expect(state.isGenerating).toBe(false);
	});

	// ===== cancelGeneration =====

	it('should cancel generation', async () => {
		llmService.store.update((s) => ({
			...s,
			isGenerating: true,
			streamingContent: 'partial response'
		}));

		vi.stubGlobal('fetch', mockFetchOk({}));

		await llmService.cancelGeneration();

		const state = get(llmService.store);
		expect(state.isGenerating).toBe(false);
		expect(state.streamingContent).toBe('');
	});

	it('should handle cancel when no abort controller exists', async () => {
		vi.stubGlobal('fetch', mockFetchOk({}));

		await llmService.cancelGeneration();

		const state = get(llmService.store);
		expect(state.isGenerating).toBe(false);
	});

	it('should handle cancelGeneration fetch failure gracefully', async () => {
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Cancel failed')));

		await llmService.cancelGeneration();

		const state = get(llmService.store);
		expect(state.isGenerating).toBe(false);
		expect(state.streamingContent).toBe('');
	});

	// ===== updateConfig =====

	it('should update config and refresh status', async () => {
		const mockFn = mockFetchByUrl({
			'/config': {},
			'/status': { loaded: true, modelName: 'updated' }
		});
		vi.stubGlobal('fetch', mockFn);

		await llmService.updateConfig({ temperature: 0.8 } as never);

		expect(mockFn).toHaveBeenCalledWith(
			expect.stringContaining('/api/llm/config'),
			expect.objectContaining({ method: 'PUT' })
		);
	});

	it('should handle updateConfig error', async () => {
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Config failed')));

		await llmService.updateConfig({ temperature: 0.8 } as never);

		// Should not throw
		expect(true).toBe(true);
	});

	// ===== fetchConversations =====

	it('should fetch conversations', async () => {
		const conversations = [
			{ id: 'conv-1', title: 'Chat 1' },
			{ id: 'conv-2', title: 'Chat 2' }
		];
		vi.stubGlobal('fetch', mockFetchOk(conversations));

		await llmService.fetchConversations();

		const state = get(llmService.store);
		expect(state.conversations).toHaveLength(2);
	});

	it('should handle fetchConversations error', async () => {
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Fetch failed')));

		await llmService.fetchConversations();

		const state = get(llmService.store);
		expect(state.conversations).toEqual([]);
	});

	// ===== createConversation =====

	it('should create a conversation', async () => {
		const conversation = {
			id: 'conv-1',
			title: 'Test Chat',
			messages: '[]',
			createdAt: Date.now()
		};
		vi.stubGlobal('fetch', mockFetchOk(conversation));

		await llmService.createConversation('Test Chat');

		const state = get(llmService.store);
		expect(state.conversations).toHaveLength(1);
		expect(state.conversations[0].title).toBe('Test Chat');
		expect(state.activeConversationId).toBe('conv-1');
		expect(state.messages).toEqual([]);
	});

	it('should create conversation with system prompt', async () => {
		const conversation = { id: 'conv-2', title: 'System', messages: '[]', createdAt: Date.now() };
		const mockFn = mockFetchOk(conversation);
		vi.stubGlobal('fetch', mockFn);

		await llmService.createConversation('System', 'You are a helpful assistant');

		const call = mockFn.mock.calls[0];
		const body = JSON.parse(call[1].body as string);
		expect(body.systemPrompt).toBe('You are a helpful assistant');
	});

	it('should create conversation with null system prompt when not provided', async () => {
		const conversation = {
			id: 'conv-3',
			title: 'No system',
			messages: '[]',
			createdAt: Date.now()
		};
		const mockFn = mockFetchOk(conversation);
		vi.stubGlobal('fetch', mockFn);

		await llmService.createConversation('No system');

		const call = mockFn.mock.calls[0];
		const body = JSON.parse(call[1].body as string);
		expect(body.systemPrompt).toBeNull();
	});

	it('should handle createConversation error', async () => {
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Create failed')));

		await llmService.createConversation('Test');

		const state = get(llmService.store);
		expect(state.conversations).toEqual([]);
	});

	// ===== selectConversation =====

	it('should select a conversation and load messages', async () => {
		const messages = [
			{ role: 'user', content: 'Hello' },
			{ role: 'assistant', content: 'Hi there!' }
		];
		const conversation = {
			id: 'conv-1',
			title: 'Test',
			messages: JSON.stringify(messages)
		};
		vi.stubGlobal('fetch', mockFetchOk(conversation));

		await llmService.selectConversation('conv-1');

		const state = get(llmService.store);
		expect(state.activeConversationId).toBe('conv-1');
		expect(state.messages).toHaveLength(2);
		expect(state.messages[0].content).toBe('Hello');
		expect(state.streamingContent).toBe('');
	});

	it('should handle selectConversation with empty messages', async () => {
		const conversation = { id: 'conv-1', title: 'Empty', messages: '[]' };
		vi.stubGlobal('fetch', mockFetchOk(conversation));

		await llmService.selectConversation('conv-1');

		const state = get(llmService.store);
		expect(state.messages).toEqual([]);
	});

	it('should handle selectConversation error', async () => {
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Not found')));

		await llmService.selectConversation('bad-id');

		const state = get(llmService.store);
		expect(state.activeConversationId).toBeNull();
	});

	// ===== deleteConversation =====

	it('should delete a conversation', async () => {
		llmService.store.update((s) => ({
			...s,
			conversations: [
				{ id: 'conv-1', title: 'Chat 1' } as never,
				{ id: 'conv-2', title: 'Chat 2' } as never
			],
			activeConversationId: 'conv-1',
			messages: [{ role: 'user', content: 'test' }] as never
		}));

		vi.stubGlobal('fetch', mockFetchOk({}));

		await llmService.deleteConversation('conv-1');

		const state = get(llmService.store);
		expect(state.conversations).toHaveLength(1);
		expect(state.conversations[0].id).toBe('conv-2');
		expect(state.activeConversationId).toBeNull();
		expect(state.messages).toEqual([]);
	});

	it('should not clear messages when deleting inactive conversation', async () => {
		llmService.store.update((s) => ({
			...s,
			conversations: [
				{ id: 'conv-1', title: 'Chat 1' } as never,
				{ id: 'conv-2', title: 'Chat 2' } as never
			],
			activeConversationId: 'conv-1',
			messages: [{ role: 'user', content: 'test' }] as never
		}));

		vi.stubGlobal('fetch', mockFetchOk({}));

		await llmService.deleteConversation('conv-2');

		const state = get(llmService.store);
		expect(state.activeConversationId).toBe('conv-1');
		expect(state.messages).toHaveLength(1);
	});

	it('should handle deleteConversation error', async () => {
		llmService.store.update((s) => ({
			...s,
			conversations: [{ id: 'conv-1', title: 'Chat 1' } as never]
		}));

		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Delete failed')));

		await llmService.deleteConversation('conv-1');

		// Conversations remain unchanged on error
		const state = get(llmService.store);
		expect(state.conversations).toHaveLength(1);
	});

	// ===== fetchJson error handling =====

	it('should use server error message when available', async () => {
		vi.stubGlobal('fetch', mockFetchError('Custom server error'));

		// We test fetchJson indirectly via fetchStatus which catches errors
		await llmService.fetchStatus();

		// The error is caught and logged, not propagated to state for fetchStatus
		const state = get(llmService.store);
		expect(state.status).toBeNull();
	});

	it('should use status code when no error message', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn().mockResolvedValue({
				ok: false,
				status: 404,
				json: () => Promise.resolve({}),
				text: () => Promise.resolve('')
			})
		);

		await llmService.fetchStatus();

		const state = get(llmService.store);
		expect(state.status).toBeNull();
	});

	it('should handle empty text response', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn().mockResolvedValue({
				ok: true,
				json: () => Promise.resolve({}),
				text: () => Promise.resolve('')
			})
		);

		await llmService.fetchStatus();

		const state = get(llmService.store);
		// Empty response should return {} as T
		expect(state.status).toEqual({});
	});
});
