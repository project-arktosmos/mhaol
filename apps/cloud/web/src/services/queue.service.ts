import { writable, get } from 'svelte/store';
import { fetchRaw } from '$transport/fetch-helpers';
import type { QueueTask, QueueEvent, QueueTaskStatus, QueueState } from '$types/queue.type';

const initialState: QueueState = {
	tasks: [],
	connected: false
};

class QueueService {
	public store = writable<QueueState>(initialState);
	private abortController: AbortController | null = null;
	private waiters = new Map<
		string,
		{ resolve: (task: QueueTask) => void; reject: (err: Error) => void; cleanup?: () => void }
	>();

	async fetchTasks(status?: QueueTaskStatus, taskType?: string): Promise<void> {
		try {
			const params = new URLSearchParams();
			if (status) params.set('status', status);
			if (taskType) params.set('taskType', taskType);
			const qs = params.toString();
			const res = await fetchRaw(`/api/queue/tasks${qs ? `?${qs}` : ''}`);
			if (!res.ok) return;
			const tasks: QueueTask[] = await res.json();
			this.store.update((s) => ({ ...s, tasks }));
		} catch {
			// best-effort
		}
	}

	async createTask(taskType: string, payload: Record<string, unknown>): Promise<QueueTask | null> {
		try {
			const res = await fetchRaw('/api/queue/tasks', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ taskType, payload })
			});
			if (!res.ok) return null;
			const task: QueueTask = await res.json();
			this.store.update((s) => ({
				...s,
				tasks: [task, ...s.tasks]
			}));
			return task;
		} catch {
			return null;
		}
	}

	async cancelTask(id: string): Promise<boolean> {
		try {
			const res = await fetchRaw(`/api/queue/tasks/${id}`, { method: 'DELETE' });
			return res.ok || res.status === 204;
		} catch {
			return false;
		}
	}

	async clearCompleted(): Promise<void> {
		const state = get(this.store);
		const terminal = state.tasks.filter(
			(t) => t.status === 'completed' || t.status === 'failed' || t.status === 'cancelled'
		);
		await Promise.all(terminal.map((t) => this.cancelTask(t.id)));
		this.store.update((s) => ({
			...s,
			tasks: s.tasks.filter(
				(t) => t.status !== 'completed' && t.status !== 'failed' && t.status !== 'cancelled'
			)
		}));
	}

	subscribe(): void {
		if (this.abortController) return;
		this.abortController = new AbortController();
		this.store.update((s) => ({ ...s, connected: true }));
		this.connectSse(this.abortController.signal);
	}

	unsubscribe(): void {
		if (this.abortController) {
			this.abortController.abort();
			this.abortController = null;
		}
		this.store.update((s) => ({ ...s, connected: false }));
	}

	waitForTask(id: string, timeoutMs = 30000): Promise<QueueTask> {
		// Check if already complete
		const state = get(this.store);
		const existing = state.tasks.find((t) => t.id === id);
		if (existing && (existing.status === 'completed' || existing.status === 'failed')) {
			return Promise.resolve(existing);
		}

		return new Promise<QueueTask>((resolve, reject) => {
			// Poll as fallback in case SSE events are missed
			const pollInterval = setInterval(async () => {
				try {
					const res = await fetchRaw(`/api/queue/tasks/${id}`);
					if (!res.ok) return;
					const task: QueueTask = await res.json();
					if (task.status === 'completed' || task.status === 'failed') {
						this.resolveWaiter(task);
					}
				} catch {
					// best-effort polling
				}
			}, 3000);

			const timeout = setTimeout(() => {
				this.rejectWaiter(id, new Error(`Task ${id} timed out after ${timeoutMs}ms`));
			}, timeoutMs);

			const cleanup = () => {
				clearInterval(pollInterval);
				clearTimeout(timeout);
			};

			this.waiters.set(id, { resolve, reject, cleanup });
		});
	}

	private async connectSse(signal: AbortSignal): Promise<void> {
		try {
			const response = await fetchRaw('/api/queue/subscribe', { signal });
			if (!response.ok || !response.body) return;

			const reader = response.body.getReader();
			const decoder = new TextDecoder();
			let buffer = '';

			while (!signal.aborted) {
				const { done, value } = await reader.read();
				if (done) break;

				buffer += decoder.decode(value, { stream: true });
				const lines = buffer.split('\n');
				buffer = lines.pop() || '';

				for (const line of lines) {
					if (line.startsWith('data: ')) {
						try {
							const event = JSON.parse(line.slice(6)) as QueueEvent;
							this.handleEvent(event);
						} catch {
							// ignore parse errors
						}
					}
				}
			}
		} catch (err) {
			if ((err as Error).name !== 'AbortError') {
				console.error('[queue] SSE connection error:', err);
			}
		} finally {
			if (!signal.aborted) {
				this.store.update((s) => ({ ...s, connected: false }));
				// Reconnect after a delay
				setTimeout(() => {
					if (this.abortController && !this.abortController.signal.aborted) {
						this.connectSse(this.abortController.signal);
					}
				}, 3000);
			}
		}
	}

	private handleEvent(event: QueueEvent): void {
		switch (event.type) {
			case 'taskCreated':
				this.store.update((s) => ({
					...s,
					tasks: [event.task, ...s.tasks.filter((t) => t.id !== event.task.id)]
				}));
				break;

			case 'taskStarted':
			case 'taskCompleted':
			case 'taskFailed':
				this.store.update((s) => ({
					...s,
					tasks: s.tasks.map((t) => (t.id === event.task.id ? event.task : t))
				}));
				if (event.type === 'taskCompleted' || event.type === 'taskFailed') {
					this.resolveWaiter(event.task);
				}
				break;

			case 'taskProgress':
				this.store.update((s) => ({
					...s,
					tasks: s.tasks.map((t) => (t.id === event.id ? { ...t, progress: event.progress } : t))
				}));
				break;

			case 'taskCancelled':
				this.store.update((s) => ({
					...s,
					tasks: s.tasks.map((t) =>
						t.id === event.id ? { ...t, status: 'cancelled' as const } : t
					)
				}));
				this.rejectWaiter(event.id, new Error('Task cancelled'));
				break;

			case 'taskRemoved':
				this.store.update((s) => ({
					...s,
					tasks: s.tasks.filter((t) => t.id !== event.id)
				}));
				this.rejectWaiter(event.id, new Error('Task removed'));
				break;
		}
	}

	private resolveWaiter(task: QueueTask): void {
		const waiter = this.waiters.get(task.id);
		if (waiter) {
			this.waiters.delete(task.id);
			waiter.cleanup?.();
			this.store.update((s) => ({
				...s,
				tasks: s.tasks.map((t) => (t.id === task.id ? task : t))
			}));
			waiter.resolve(task);
		}
	}

	private rejectWaiter(id: string, error: Error): void {
		const waiter = this.waiters.get(id);
		if (waiter) {
			this.waiters.delete(id);
			waiter.cleanup?.();
			waiter.reject(error);
		}
	}
}

export const queueService = new QueueService();
