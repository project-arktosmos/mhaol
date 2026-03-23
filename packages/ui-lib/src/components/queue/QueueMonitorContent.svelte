<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { queueService } from 'ui-lib/services/queue.service';
	import classNames from 'classnames';
	import type { QueueTask } from 'ui-lib/types/queue.type';

	const store = queueService.store;

	let tasks = $derived($store.tasks);
	let connected = $derived($store.connected);

	onMount(() => {
		queueService.fetchTasks();
		queueService.subscribe();
	});

	onDestroy(() => {
		queueService.unsubscribe();
	});

	function statusBadgeClass(status: string): string {
		return classNames('badge badge-sm', {
			'badge-warning': status === 'pending',
			'badge-info': status === 'running',
			'badge-success': status === 'completed',
			'badge-error': status === 'failed',
			'badge-ghost': status === 'cancelled'
		});
	}

	function formatDuration(task: QueueTask): string {
		const start = task.startedAt ?? task.createdAt;
		const end = task.completedAt ?? new Date().toISOString();
		const ms = new Date(end).getTime() - new Date(start).getTime();
		if (ms < 1000) return `${ms}ms`;
		if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
		return `${Math.floor(ms / 60000)}m ${Math.floor((ms % 60000) / 1000)}s`;
	}

	function truncate(text: string, max: number): string {
		return text.length > max ? text.slice(0, max) + '...' : text;
	}
</script>

<div class="flex flex-col gap-4">
	<div class="flex items-center justify-between">
		<div class="flex items-center gap-2">
			<span class={classNames('badge badge-xs', connected ? 'badge-success' : 'badge-error')}
			></span>
			<span class="text-xs text-base-content/50">{connected ? 'Connected' : 'Disconnected'}</span>
		</div>
		<div class="flex gap-2">
			<button class="btn btn-ghost btn-xs" onclick={() => queueService.fetchTasks()}>
				Refresh
			</button>
			<button class="btn btn-ghost btn-xs" onclick={() => queueService.clearCompleted()}>
				Clear Completed
			</button>
		</div>
	</div>

	{#if tasks.length === 0}
		<p class="py-8 text-center text-sm text-base-content/50">No tasks in queue</p>
	{:else}
		<div class="overflow-x-auto">
			<table class="table table-sm">
				<thead>
					<tr>
						<th>Type</th>
						<th>Status</th>
						<th>Payload</th>
						<th>Created</th>
						<th>Duration</th>
						<th></th>
					</tr>
				</thead>
				<tbody>
					{#each tasks as task (task.id)}
						<tr>
							<td class="font-mono text-xs">{task.taskType}</td>
							<td><span class={statusBadgeClass(task.status)}>{task.status}</span></td>
							<td class="max-w-xs text-xs">
								{truncate(JSON.stringify(task.payload), 80)}
							</td>
							<td class="text-xs">{new Date(task.createdAt).toLocaleTimeString()}</td>
							<td class="text-xs">{formatDuration(task)}</td>
							<td>
								{#if task.status === 'pending' || task.status === 'running'}
									<button
										class="btn btn-ghost btn-xs"
										onclick={() => queueService.cancelTask(task.id)}
									>
										Cancel
									</button>
								{/if}
							</td>
						</tr>
						{#if task.error}
							<tr>
								<td colspan="6" class="text-xs text-error">{task.error}</td>
							</tr>
						{/if}
						{#if task.result}
							<tr>
								<td colspan="6" class="text-xs text-base-content/70">
									{truncate(JSON.stringify(task.result), 120)}
								</td>
							</tr>
						{/if}
					{/each}
				</tbody>
			</table>
		</div>
	{/if}
</div>
