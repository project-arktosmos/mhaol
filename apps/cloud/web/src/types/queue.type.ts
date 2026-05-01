export type QueueTaskStatus = 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';

export interface QueueTask {
	id: string;
	taskType: string;
	status: QueueTaskStatus;
	payload: Record<string, unknown>;
	result: Record<string, unknown> | null;
	error: string | null;
	progress: Record<string, unknown> | null;
	createdAt: string;
	startedAt: string | null;
	completedAt: string | null;
}

export type QueueEvent =
	| { type: 'taskCreated'; task: QueueTask }
	| { type: 'taskStarted'; task: QueueTask }
	| { type: 'taskProgress'; id: string; progress: Record<string, unknown> }
	| { type: 'taskCompleted'; task: QueueTask }
	| { type: 'taskFailed'; task: QueueTask }
	| { type: 'taskCancelled'; id: string }
	| { type: 'taskRemoved'; id: string };

export interface QueueState {
	tasks: QueueTask[];
	connected: boolean;
}
