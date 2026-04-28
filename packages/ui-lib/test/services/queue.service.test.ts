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

describe('QueueService', () => {
	let queueService: (typeof import('../../src/services/queue.service'))['queueService'];

	beforeEach(async () => {
		vi.resetModules();
		vi.stubGlobal('fetch', vi.fn());
		const mod = await import('../../src/services/queue.service');
		queueService = mod.queueService;
	});

	afterEach(() => {
		queueService.unsubscribe();
		vi.unstubAllGlobals();
	});

	function makeTask(overrides: Record<string, unknown> = {}) {
		return {
			id: 'task-1',
			taskType: 'job',
			status: 'pending',
			payload: {},
			result: null,
			error: null,
			progress: null,
			createdAt: '2026-01-01T00:00:00Z',
			startedAt: null,
			completedAt: null,
			...overrides
		};
	}

	// ===== Initial state =====

	it('should have correct initial state', () => {
		const state = get(queueService.store);
		expect(state.tasks).toEqual([]);
		expect(state.connected).toBe(false);
	});

	// ===== fetchTasks =====

	it('should fetch tasks and update store', async () => {
		const tasks = [makeTask({ id: 'task-1' }), makeTask({ id: 'task-2' })];
		vi.stubGlobal('fetch', mockFetchOk(tasks));

		await queueService.fetchTasks();

		const state = get(queueService.store);
		expect(state.tasks).toHaveLength(2);
	});

	it('should fetch tasks with status filter', async () => {
		const mockFn = mockFetchOk([]);
		vi.stubGlobal('fetch', mockFn);

		await queueService.fetchTasks('pending');

		expect(mockFn).toHaveBeenCalledWith(expect.stringContaining('status=pending'));
	});

	it('should fetch tasks with taskType filter', async () => {
		const mockFn = mockFetchOk([]);
		vi.stubGlobal('fetch', mockFn);

		await queueService.fetchTasks(undefined, 'job');

		expect(mockFn).toHaveBeenCalledWith(expect.stringContaining('taskType=job'));
	});

	it('should handle fetchTasks failure silently', async () => {
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Network error')));

		await queueService.fetchTasks();

		const state = get(queueService.store);
		expect(state.tasks).toEqual([]);
	});

	it('should handle fetchTasks non-ok response', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn().mockResolvedValue({
				ok: false,
				status: 500,
				json: () => Promise.resolve({}),
				body: null
			})
		);

		await queueService.fetchTasks();

		const state = get(queueService.store);
		expect(state.tasks).toEqual([]);
	});

	// ===== createTask =====

	it('should create a task and prepend to store', async () => {
		const newTask = makeTask({ id: 'new-task' });
		vi.stubGlobal('fetch', mockFetchOk(newTask));

		const result = await queueService.createTask('job', { prompt: 'hello' });

		expect(result).not.toBeNull();
		expect(result!.id).toBe('new-task');

		const state = get(queueService.store);
		expect(state.tasks).toHaveLength(1);
		expect(state.tasks[0].id).toBe('new-task');
	});

	it('should prepend new task before existing tasks', async () => {
		// First populate with an existing task
		vi.stubGlobal('fetch', mockFetchOk([makeTask({ id: 'existing' })]));
		await queueService.fetchTasks();

		// Now create a new one
		const newTask = makeTask({ id: 'new-task' });
		vi.stubGlobal('fetch', mockFetchOk(newTask));

		await queueService.createTask('job', {});

		const state = get(queueService.store);
		expect(state.tasks[0].id).toBe('new-task');
		expect(state.tasks[1].id).toBe('existing');
	});

	it('should return null on createTask failure', async () => {
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Failed')));

		const result = await queueService.createTask('job', {});

		expect(result).toBeNull();
	});

	it('should return null on createTask non-ok response', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn().mockResolvedValue({
				ok: false,
				status: 400,
				json: () => Promise.resolve({}),
				body: null
			})
		);

		const result = await queueService.createTask('job', {});

		expect(result).toBeNull();
	});

	// ===== cancelTask =====

	it('should cancel a task and return true', async () => {
		const mockFn = vi.fn().mockResolvedValue({ ok: true, status: 200 });
		vi.stubGlobal('fetch', mockFn);

		const result = await queueService.cancelTask('task-1');

		expect(result).toBe(true);
		expect(mockFn).toHaveBeenCalledWith(
			expect.stringContaining('/api/queue/tasks/task-1'),
			expect.objectContaining({ method: 'DELETE' })
		);
	});

	it('should return true for 204 response on cancel', async () => {
		vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: false, status: 204 }));

		const result = await queueService.cancelTask('task-1');

		expect(result).toBe(true);
	});

	it('should return false on cancelTask failure', async () => {
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Failed')));

		const result = await queueService.cancelTask('task-1');

		expect(result).toBe(false);
	});

	// ===== clearCompleted =====

	it('should remove completed, failed, and cancelled tasks', async () => {
		const tasks = [
			makeTask({ id: 'running', status: 'running' }),
			makeTask({ id: 'completed', status: 'completed' }),
			makeTask({ id: 'failed', status: 'failed' }),
			makeTask({ id: 'cancelled', status: 'cancelled' }),
			makeTask({ id: 'pending', status: 'pending' })
		];
		vi.stubGlobal('fetch', mockFetchOk(tasks));
		await queueService.fetchTasks();

		// Mock cancel calls
		vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: true, status: 200 }));

		await queueService.clearCompleted();

		const state = get(queueService.store);
		expect(state.tasks).toHaveLength(2);
		const ids = state.tasks.map((t) => t.id);
		expect(ids).toContain('running');
		expect(ids).toContain('pending');
	});

	// ===== subscribe / unsubscribe =====

	it('should set connected to true on subscribe', () => {
		vi.stubGlobal(
			'fetch',
			vi.fn().mockResolvedValue({
				ok: true,
				body: {
					getReader: () => ({
						read: vi.fn().mockResolvedValue({ done: true, value: undefined })
					})
				}
			})
		);

		queueService.subscribe();

		const state = get(queueService.store);
		expect(state.connected).toBe(true);
	});

	it('should not create duplicate subscriptions', () => {
		const mockFn = vi.fn().mockResolvedValue({
			ok: true,
			body: {
				getReader: () => ({
					read: vi.fn().mockResolvedValue({ done: true, value: undefined })
				})
			}
		});
		vi.stubGlobal('fetch', mockFn);

		queueService.subscribe();
		queueService.subscribe();

		// Only one SSE connection should be made
		expect(mockFn).toHaveBeenCalledTimes(1);
	});

	it('should set connected to false on unsubscribe', () => {
		vi.stubGlobal(
			'fetch',
			vi.fn().mockResolvedValue({
				ok: true,
				body: {
					getReader: () => ({
						read: vi.fn().mockResolvedValue({ done: true, value: undefined })
					})
				}
			})
		);

		queueService.subscribe();
		queueService.unsubscribe();

		const state = get(queueService.store);
		expect(state.connected).toBe(false);
	});

	// ===== SSE event handling =====

	it('should handle taskCreated SSE event', async () => {
		const newTask = makeTask({ id: 'sse-task', status: 'pending' });
		const encoder = new TextEncoder();
		const chunk = encoder.encode(
			`data: ${JSON.stringify({ type: 'taskCreated', task: newTask })}\n\n`
		);

		let readCount = 0;
		vi.stubGlobal(
			'fetch',
			vi.fn().mockResolvedValue({
				ok: true,
				body: {
					getReader: () => ({
						read: vi.fn().mockImplementation(async () => {
							if (readCount === 0) {
								readCount++;
								return { done: false, value: chunk };
							}
							return { done: true, value: undefined };
						})
					})
				}
			})
		);

		queueService.subscribe();

		// Wait for the SSE processing
		await vi.waitFor(() => {
			const state = get(queueService.store);
			expect(state.tasks).toHaveLength(1);
		});

		const state = get(queueService.store);
		expect(state.tasks[0].id).toBe('sse-task');
	});

	it('should handle taskCompleted SSE event and update existing task', async () => {
		// Pre-populate with a running task
		vi.stubGlobal('fetch', mockFetchOk([makeTask({ id: 'task-1', status: 'running' })]));
		await queueService.fetchTasks();

		const completedTask = makeTask({ id: 'task-1', status: 'completed', result: { answer: '42' } });
		const encoder = new TextEncoder();
		const chunk = encoder.encode(
			`data: ${JSON.stringify({ type: 'taskCompleted', task: completedTask })}\n\n`
		);

		let readCount = 0;
		vi.stubGlobal(
			'fetch',
			vi.fn().mockResolvedValue({
				ok: true,
				body: {
					getReader: () => ({
						read: vi.fn().mockImplementation(async () => {
							if (readCount === 0) {
								readCount++;
								return { done: false, value: chunk };
							}
							return { done: true, value: undefined };
						})
					})
				}
			})
		);

		queueService.subscribe();

		await vi.waitFor(() => {
			const state = get(queueService.store);
			expect(state.tasks[0].status).toBe('completed');
		});
	});

	it('should handle taskProgress SSE event', async () => {
		// Pre-populate
		vi.stubGlobal('fetch', mockFetchOk([makeTask({ id: 'task-1', status: 'running' })]));
		await queueService.fetchTasks();

		const encoder = new TextEncoder();
		const chunk = encoder.encode(
			`data: ${JSON.stringify({ type: 'taskProgress', id: 'task-1', progress: { percent: 50 } })}\n\n`
		);

		let readCount = 0;
		vi.stubGlobal(
			'fetch',
			vi.fn().mockResolvedValue({
				ok: true,
				body: {
					getReader: () => ({
						read: vi.fn().mockImplementation(async () => {
							if (readCount === 0) {
								readCount++;
								return { done: false, value: chunk };
							}
							return { done: true, value: undefined };
						})
					})
				}
			})
		);

		queueService.subscribe();

		await vi.waitFor(() => {
			const state = get(queueService.store);
			expect(state.tasks[0].progress).toEqual({ percent: 50 });
		});
	});

	it('should handle taskRemoved SSE event', async () => {
		// Pre-populate
		vi.stubGlobal('fetch', mockFetchOk([makeTask({ id: 'task-1' })]));
		await queueService.fetchTasks();
		expect(get(queueService.store).tasks).toHaveLength(1);

		const encoder = new TextEncoder();
		const chunk = encoder.encode(
			`data: ${JSON.stringify({ type: 'taskRemoved', id: 'task-1' })}\n\n`
		);

		let readCount = 0;
		vi.stubGlobal(
			'fetch',
			vi.fn().mockResolvedValue({
				ok: true,
				body: {
					getReader: () => ({
						read: vi.fn().mockImplementation(async () => {
							if (readCount === 0) {
								readCount++;
								return { done: false, value: chunk };
							}
							return { done: true, value: undefined };
						})
					})
				}
			})
		);

		queueService.subscribe();

		await vi.waitFor(() => {
			const state = get(queueService.store);
			expect(state.tasks).toHaveLength(0);
		});
	});

	// ===== waitForTask =====

	it('should resolve immediately if task is already completed', async () => {
		vi.stubGlobal(
			'fetch',
			mockFetchOk([makeTask({ id: 'task-1', status: 'completed', result: { done: true } })])
		);
		await queueService.fetchTasks();

		const result = await queueService.waitForTask('task-1');

		expect(result.status).toBe('completed');
	});

	it('should resolve immediately if task is already failed', async () => {
		vi.stubGlobal(
			'fetch',
			mockFetchOk([makeTask({ id: 'task-1', status: 'failed', error: 'boom' })])
		);
		await queueService.fetchTasks();

		const result = await queueService.waitForTask('task-1');

		expect(result.status).toBe('failed');
	});
});
