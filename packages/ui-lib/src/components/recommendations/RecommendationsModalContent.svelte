<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import classNames from 'classnames';
	import { queueService } from 'ui-lib/services/queue.service';
	import { recommendationsService } from 'ui-lib/services/recommendations.service';
	import type { QueueTask } from 'ui-lib/types/queue.type';
	import type { RecommendationRow, RecommendationsStatus } from 'ui-lib/types/recommendations.type';
	import type { DisplayTMDBMovie } from 'addons/tmdb/types';

	interface Props {
		pinnedMovies: DisplayTMDBMovie[];
		favoritedMovies: DisplayTMDBMovie[];
		libraryMovieTmdbIds: number[];
	}

	let { pinnedMovies, favoritedMovies, libraryMovieTmdbIds }: Props = $props();

	let status = $state<RecommendationsStatus | null>(null);
	let enqueueing = $state(false);
	let tasks = $state<QueueTask[]>([]);
	let expandedTaskId = $state<string | null>(null);
	let expandedRecs = $state<Map<string, RecommendationRow[]>>(new Map());
	let loadingRecs = $state<Set<string>>(new Set());

	const queueStore = queueService.store;

	let recTasks = $derived($queueStore.tasks.filter((t) => t.taskType === 'recommendations:fetch'));

	let connected = $derived($queueStore.connected);

	let allTmdbIds = $derived(() => {
		const ids = new Set<number>();
		for (const m of pinnedMovies) ids.add(m.id);
		for (const m of favoritedMovies) ids.add(m.id);
		for (const id of libraryMovieTmdbIds) ids.add(id);
		return ids;
	});

	let pendingCount = $derived(recTasks.filter((t) => t.status === 'pending').length);
	let runningCount = $derived(recTasks.filter((t) => t.status === 'running').length);
	let completedCount = $derived(recTasks.filter((t) => t.status === 'completed').length);
	let failedCount = $derived(recTasks.filter((t) => t.status === 'failed').length);

	async function loadStatus() {
		try {
			status = await recommendationsService.getStatus();
		} catch {
			/* best-effort */
		}
	}

	async function enqueueAll() {
		const ids = allTmdbIds();
		if (ids.size === 0) return;
		enqueueing = true;
		try {
			const items = [...ids].map((tmdbId) => ({ tmdbId, mediaType: 'movie' as const }));
			await recommendationsService.bulkEnqueue(items);
			await loadStatus();
			await queueService.fetchTasks(undefined, 'recommendations:fetch');
		} catch {
			/* best-effort */
		} finally {
			enqueueing = false;
		}
	}

	async function enqueuePinned() {
		if (pinnedMovies.length === 0) return;
		enqueueing = true;
		try {
			const items = pinnedMovies.map((m) => ({ tmdbId: m.id, mediaType: 'movie' as const }));
			await recommendationsService.bulkEnqueue(items);
			await queueService.fetchTasks(undefined, 'recommendations:fetch');
		} catch {
			/* best-effort */
		} finally {
			enqueueing = false;
		}
	}

	async function enqueueFavorited() {
		if (favoritedMovies.length === 0) return;
		enqueueing = true;
		try {
			const items = favoritedMovies.map((m) => ({ tmdbId: m.id, mediaType: 'movie' as const }));
			await recommendationsService.bulkEnqueue(items);
			await queueService.fetchTasks(undefined, 'recommendations:fetch');
		} catch {
			/* best-effort */
		} finally {
			enqueueing = false;
		}
	}

	async function enqueueLibrary() {
		if (libraryMovieTmdbIds.length === 0) return;
		enqueueing = true;
		try {
			const items = libraryMovieTmdbIds.map((tmdbId) => ({
				tmdbId,
				mediaType: 'movie' as const
			}));
			await recommendationsService.bulkEnqueue(items);
			await queueService.fetchTasks(undefined, 'recommendations:fetch');
		} catch {
			/* best-effort */
		} finally {
			enqueueing = false;
		}
	}

	async function toggleExpand(task: QueueTask) {
		const key = task.id;
		if (expandedTaskId === key) {
			expandedTaskId = null;
			return;
		}
		expandedTaskId = key;
		if (task.status !== 'completed') return;

		const tmdbId = task.payload.tmdbId as number;
		const mediaType = (task.payload.mediaType as string) ?? 'movie';
		if (expandedRecs.has(key)) return;

		loadingRecs = new Set([...loadingRecs, key]);
		try {
			const recs = await recommendationsService.getForSource(tmdbId, mediaType);
			expandedRecs = new Map([...expandedRecs, [key, recs]]);
		} catch {
			/* best-effort */
		} finally {
			const next = new Set(loadingRecs);
			next.delete(key);
			loadingRecs = next;
		}
	}

	function statusBadgeClass(s: string): string {
		return classNames('badge badge-sm', {
			'badge-warning': s === 'pending',
			'badge-info': s === 'running',
			'badge-success': s === 'completed',
			'badge-error': s === 'failed',
			'badge-ghost': s === 'cancelled'
		});
	}

	function taskTitle(task: QueueTask): string {
		const id = task.payload.tmdbId as number;
		const result = task.result as Record<string, unknown> | null;
		if (result?.title) return `${result.title} (${id})`;
		return `TMDB #${id}`;
	}

	function formatTime(iso: string): string {
		return new Date(iso).toLocaleTimeString();
	}

	function parseRecData(row: RecommendationRow): Record<string, unknown> | null {
		try {
			return row.data as Record<string, unknown>;
		} catch {
			return null;
		}
	}

	onMount(() => {
		loadStatus();
		queueService.fetchTasks(undefined, 'recommendations:fetch');
		queueService.subscribe();
	});

	onDestroy(() => {
		queueService.unsubscribe();
	});
</script>

<div class="flex max-h-[80vh] flex-col gap-4 overflow-hidden">
	<div class="flex items-center justify-between">
		<h2 class="text-lg font-bold">Recommendations Queue</h2>
		<div class="flex items-center gap-2">
			<span class={classNames('badge badge-xs', connected ? 'badge-success' : 'badge-error')}
			></span>
			<span class="text-xs text-base-content/50">{connected ? 'Live' : 'Offline'}</span>
		</div>
	</div>

	<div class="flex flex-wrap gap-2">
		<button
			class="btn btn-sm btn-primary"
			disabled={enqueueing || allTmdbIds().size === 0}
			onclick={enqueueAll}
		>
			{#if enqueueing}
				<span class="loading loading-xs loading-spinner"></span>
			{/if}
			Enqueue All ({allTmdbIds().size})
		</button>
		<button
			class="btn btn-outline btn-sm"
			disabled={enqueueing || pinnedMovies.length === 0}
			onclick={enqueuePinned}
		>
			Pinned ({pinnedMovies.length})
		</button>
		<button
			class="btn btn-outline btn-sm"
			disabled={enqueueing || favoritedMovies.length === 0}
			onclick={enqueueFavorited}
		>
			Favorites ({favoritedMovies.length})
		</button>
		<button
			class="btn btn-outline btn-sm"
			disabled={enqueueing || libraryMovieTmdbIds.length === 0}
			onclick={enqueueLibrary}
		>
			Library ({libraryMovieTmdbIds.length})
		</button>
	</div>

	{#if status}
		<div class="flex flex-wrap gap-3 text-xs">
			<span class="badge badge-sm badge-warning">Pending: {status.pending}</span>
			<span class="badge badge-sm badge-info">Running: {status.running}</span>
			<span class="badge badge-sm badge-success">Completed: {status.completed}</span>
			<span class="badge badge-sm badge-error">Failed: {status.failed}</span>
			<span class="badge badge-ghost badge-sm">Total: {status.total}</span>
		</div>
	{/if}

	{#if recTasks.length > 0}
		<div class="flex items-center justify-between text-xs text-base-content/50">
			<span>
				{pendingCount} pending, {runningCount} running, {completedCount} completed, {failedCount} failed
			</span>
			<button class="btn btn-ghost btn-xs" onclick={() => queueService.clearCompleted()}>
				Clear Done
			</button>
		</div>
	{/if}

	<div class="flex-1 overflow-y-auto">
		{#if recTasks.length === 0}
			<p class="py-8 text-center text-sm text-base-content/50">
				No recommendation tasks. Enqueue movies above to get started.
			</p>
		{:else}
			<div class="flex flex-col gap-1">
				{#each recTasks as task (task.id)}
					{@const isExpanded = expandedTaskId === task.id}
					{@const recs = expandedRecs.get(task.id)}
					{@const isLoadingRecs = loadingRecs.has(task.id)}
					<div class="rounded-lg bg-base-200 p-2">
						<button
							class="flex w-full items-center gap-2 text-left"
							onclick={() => toggleExpand(task)}
						>
							<span class={statusBadgeClass(task.status)}>{task.status}</span>
							<span class="flex-1 truncate text-sm font-medium">{taskTitle(task)}</span>
							{#if task.status === 'running'}
								<span class="loading loading-xs loading-spinner"></span>
							{/if}
							{#if task.result}
								{@const count = (task.result as Record<string, unknown>).count}
								{#if count !== undefined}
									<span class="badge badge-ghost badge-xs">{count} recs</span>
								{/if}
							{/if}
							<span class="text-xs text-base-content/40">{formatTime(task.createdAt)}</span>
							<span class="text-xs text-base-content/30">{isExpanded ? '▲' : '▼'}</span>
						</button>

						{#if task.error}
							<p class="mt-1 text-xs text-error">{task.error}</p>
						{/if}

						{#if isExpanded}
							<div class="mt-2 border-t border-base-300 pt-2">
								<div class="mb-2 text-xs text-base-content/60">
									<p><strong>Task ID:</strong> {task.id}</p>
									<p><strong>Payload:</strong> {JSON.stringify(task.payload)}</p>
									{#if task.result}
										<p><strong>Result:</strong> {JSON.stringify(task.result)}</p>
									{/if}
									{#if task.startedAt}
										<p><strong>Started:</strong> {task.startedAt}</p>
									{/if}
									{#if task.completedAt}
										<p><strong>Completed:</strong> {task.completedAt}</p>
									{/if}
								</div>

								{#if task.status === 'completed'}
									{#if isLoadingRecs}
										<div class="flex justify-center py-2">
											<span class="loading loading-xs loading-spinner"></span>
										</div>
									{:else if recs && recs.length > 0}
										<div class="max-h-60 overflow-y-auto">
											<table class="table table-xs">
												<thead>
													<tr>
														<th>TMDB ID</th>
														<th>Title</th>
														<th>Genres</th>
														<th>Type</th>
														<th>Fetched</th>
													</tr>
												</thead>
												<tbody>
													{#each recs as rec (rec.id)}
														{@const parsed = parseRecData(rec)}
														<tr>
															<td class="font-mono">{rec.recommendedTmdbId}</td>
															<td class="max-w-40 truncate">
																{rec.title ?? parsed?.title ?? parsed?.name ?? '—'}
															</td>
															<td class="max-w-48 truncate text-xs">
																{rec.genres ?? '—'}
															</td>
															<td>{rec.recommendedMediaType}</td>
															<td class="text-xs text-base-content/50">
																{new Date(rec.fetchedAt).toLocaleDateString()}
															</td>
														</tr>
														{#if parsed}
															<tr>
																<td colspan="5" class="text-xs text-base-content/40">
																	{#if parsed.overview}
																		<p class="line-clamp-2">{parsed.overview}</p>
																	{/if}
																	{#if parsed.vote_average}
																		<span>Rating: {parsed.vote_average}</span>
																	{/if}
																	{#if parsed.release_date || parsed.first_air_date}
																		<span class="ml-2">
																			Date: {parsed.release_date ?? parsed.first_air_date}
																		</span>
																	{/if}
																	{#if parsed.genre_ids}
																		<span class="ml-2">
																			Genres: {(parsed.genre_ids as number[]).join(', ')}
																		</span>
																	{/if}
																	{#if parsed.poster_path}
																		<span class="ml-2">Poster: {parsed.poster_path}</span>
																	{/if}
																	{#if parsed.backdrop_path}
																		<span class="ml-2">
																			Backdrop: {parsed.backdrop_path}
																		</span>
																	{/if}
																	{#if parsed.popularity}
																		<span class="ml-2">
																			Popularity: {parsed.popularity}
																		</span>
																	{/if}
																	{#if parsed.original_language}
																		<span class="ml-2">
																			Lang: {parsed.original_language}
																		</span>
																	{/if}
																</td>
															</tr>
														{/if}
													{/each}
												</tbody>
											</table>
										</div>
									{:else if recs}
										<p class="text-xs text-base-content/50">
											No recommendations stored for this source.
										</p>
									{/if}
								{/if}

								{#if task.status === 'pending' || task.status === 'running'}
									<button
										class="btn mt-1 btn-ghost btn-xs"
										onclick={() => queueService.cancelTask(task.id)}
									>
										Cancel
									</button>
								{/if}
							</div>
						{/if}
					</div>
				{/each}
			</div>
		{/if}
	</div>
</div>
