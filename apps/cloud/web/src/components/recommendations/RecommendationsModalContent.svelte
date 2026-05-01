<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import classNames from 'classnames';
	import { queueService } from '$services/queue.service';
	import { recommendationsService } from '$services/recommendations.service';
	import { recommendationLabelsService } from '$services/recommendation-labels.service';
	import { profileService } from '$services/profile.service';
	import type { QueueTask } from '$types/queue.type';
	import type {
		RecommendationRow,
		RecommendationsStatus,
		TopRecommendedMovie,
		TopRecommendedMovieDetail
	} from '$types/recommendations.type';
	import type { RecommendationLabel } from '$types/recommendation-label.type';
	import { getPosterUrl, getBackdropUrl } from 'addons/tmdb/transform';
	import TopRecommendedTable from './TopRecommendedTable.svelte';
	import RecommendationLabelGrid from './RecommendationLabelGrid.svelte';

	interface Props {
		mediaType: 'movie' | 'tv';
		pinnedIds: number[];
		favoritedIds: number[];
		libraryTmdbIds: number[];
	}

	let { mediaType, pinnedIds, favoritedIds, libraryTmdbIds }: Props = $props();

	let label = $derived(mediaType === 'movie' ? 'Movie' : 'TV');

	// === Queue state ===
	let status = $state<RecommendationsStatus | null>(null);
	let enqueueing = $state(false);
	let expandedTaskId = $state<string | null>(null);
	let expandedRecs = $state<Map<string, RecommendationRow[]>>(new Map());
	let loadingRecs = $state<Set<string>>(new Set());

	let topTable: ReturnType<typeof TopRecommendedTable>;

	const queueStore = queueService.store;

	let recTasks = $derived(
		$queueStore.tasks.filter(
			(t) => t.taskType === 'recommendations:fetch' && t.payload.mediaType === mediaType
		)
	);
	let connected = $derived($queueStore.connected);

	let allTmdbIds = $derived(() => {
		const ids = new Set<number>();
		for (const id of pinnedIds) ids.add(id);
		for (const id of favoritedIds) ids.add(id);
		for (const id of libraryTmdbIds) ids.add(id);
		return ids;
	});

	let pendingCount = $derived(recTasks.filter((t) => t.status === 'pending').length);
	let runningCount = $derived(recTasks.filter((t) => t.status === 'running').length);
	let completedCount = $derived(recTasks.filter((t) => t.status === 'completed').length);
	let failedCount = $derived(recTasks.filter((t) => t.status === 'failed').length);

	// === Explore state ===
	let detailMap = $state<Map<number, TopRecommendedMovieDetail>>(new Map());
	let detailLoading = $state(false);
	let selectedIndex = $state<number | null>(null);
	let selectedMovie = $state<TopRecommendedMovie | null>(null);

	let labelDefs = $state<RecommendationLabel[]>([]);
	let assignmentMap = $state<Map<string, string>>(new Map());
	let labelLoading = $state(false);
	let wallet = $state('');

	const profileStore = profileService.state;
	profileStore.subscribe((s) => {
		wallet = s.local.wallet;
	});

	let activeLabelId = $derived.by(() => {
		if (!selectedMovie) return null;
		const key = `${selectedMovie.tmdbId}:${selectedMovie.mediaType}`;
		return assignmentMap.get(key) ?? null;
	});

	let labelEmojiMap = $derived.by(() => {
		const emojiByLabelId = new Map(labelDefs.map((l) => [l.id, l.emoji]));
		const result = new Map<string, string>();
		for (const [key, labelId] of assignmentMap) {
			const emoji = emojiByLabelId.get(labelId);
			if (emoji) result.set(key, emoji);
		}
		return result;
	});

	let selectedDetail = $derived(
		selectedMovie ? (detailMap.get(selectedMovie.tmdbId) ?? null) : null
	);

	// === Queue functions ===
	async function loadStatus() {
		try {
			status = await recommendationsService.getStatus(mediaType);
		} catch {
			/* best-effort */
		}
	}

	async function enqueueAndRefresh(items: { tmdbId: number; mediaType: 'movie' | 'tv' }[]) {
		if (items.length === 0) return;
		enqueueing = true;
		try {
			await recommendationsService.bulkEnqueue(items);
			await Promise.all([
				loadStatus(),
				queueService.fetchTasks(undefined, 'recommendations:fetch')
			]);
		} catch {
			/* best-effort */
		} finally {
			enqueueing = false;
		}
	}

	function enqueueAll() {
		enqueueAndRefresh([...allTmdbIds()].map((tmdbId) => ({ tmdbId, mediaType })));
	}

	function enqueuePinned() {
		enqueueAndRefresh(pinnedIds.map((tmdbId) => ({ tmdbId, mediaType })));
	}

	function enqueueFavorited() {
		enqueueAndRefresh(favoritedIds.map((tmdbId) => ({ tmdbId, mediaType })));
	}

	function enqueueLibrary() {
		enqueueAndRefresh(libraryTmdbIds.map((tmdbId) => ({ tmdbId, mediaType })));
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

	// Refresh stats when tasks complete
	let prevCompleted = 0;
	$effect(() => {
		const c = completedCount;
		if (c > prevCompleted && prevCompleted > 0) {
			topTable?.refresh();
			loadDetails();
		}
		prevCompleted = c;
	});

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

	// === Explore functions ===
	async function loadDetails() {
		detailLoading = true;
		try {
			const movies = await recommendationsService.getTopMoviesDetail(mediaType);
			detailMap = new Map(movies.map((m) => [m.tmdbId, m]));
		} catch {
			/* best-effort */
		} finally {
			detailLoading = false;
		}
	}

	async function loadLabelDefs() {
		try {
			labelDefs = await recommendationLabelsService.getDefinitions();
		} catch {
			/* best-effort */
		}
	}

	async function loadAssignments() {
		if (!wallet) return;
		try {
			const assignments = await recommendationLabelsService.getAssignments(wallet);
			assignmentMap = new Map(
				assignments.map((a) => [`${a.recommendedTmdbId}:${a.recommendedMediaType}`, a.labelId])
			);
		} catch {
			/* best-effort */
		}
	}

	async function handleLabelClick(labelId: string) {
		if (!selectedMovie || !wallet) return;
		const key = `${selectedMovie.tmdbId}:${selectedMovie.mediaType}`;
		labelLoading = true;
		try {
			if (activeLabelId === labelId) {
				await recommendationLabelsService.removeLabel(
					wallet,
					selectedMovie.tmdbId,
					selectedMovie.mediaType
				);
				const next = new Map(assignmentMap);
				next.delete(key);
				assignmentMap = next;
			} else {
				await recommendationLabelsService.setLabel(
					wallet,
					selectedMovie.tmdbId,
					selectedMovie.mediaType,
					labelId
				);
				const next = new Map(assignmentMap);
				next.set(key, labelId);
				assignmentMap = next;
			}
		} catch {
			/* best-effort */
		} finally {
			labelLoading = false;
		}
	}

	function posterUrl(data: Record<string, unknown> | null): string | null {
		if (!data) return null;
		return getPosterUrl(data.poster_path as string | null);
	}

	function backdropUrl(data: Record<string, unknown> | null): string | null {
		if (!data) return null;
		return getBackdropUrl(data.backdrop_path as string | null);
	}

	function genres(data: Record<string, unknown> | null): string[] {
		if (!data) return [];
		const g = data.genres as Array<{ id: number; name: string }> | undefined;
		if (Array.isArray(g)) return g.map((x) => x.name);
		return [];
	}

	function handleRowClick(index: number, movie: TopRecommendedMovie) {
		selectedIndex = index;
		selectedMovie = movie;
	}

	// === Lifecycle ===
	onMount(() => {
		loadStatus();
		topTable?.refresh();
		loadDetails();
		loadLabelDefs();
		loadAssignments();
		queueService.fetchTasks(undefined, 'recommendations:fetch');
		queueService.subscribe();
	});

	onDestroy(() => {
		queueService.unsubscribe();
	});
</script>

<div class="flex max-h-[80vh] flex-col gap-4 overflow-hidden">
	<div class="flex items-center justify-between">
		<h2 class="text-lg font-bold">{label} Recommendations</h2>
		<div class="flex items-center gap-3">
			<button
				class="btn btn-ghost btn-sm"
				onclick={() => {
					topTable?.refresh();
					loadDetails();
				}}
				disabled={detailLoading}
			>
				{#if detailLoading}
					<span class="loading loading-xs loading-spinner"></span>
				{/if}
				Refresh
			</button>
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
			disabled={enqueueing || pinnedIds.length === 0}
			onclick={enqueuePinned}
		>
			Pinned ({pinnedIds.length})
		</button>
		<button
			class="btn btn-outline btn-sm"
			disabled={enqueueing || favoritedIds.length === 0}
			onclick={enqueueFavorited}
		>
			Favorites ({favoritedIds.length})
		</button>
		<button
			class="btn btn-outline btn-sm"
			disabled={enqueueing || libraryTmdbIds.length === 0}
			onclick={enqueueLibrary}
		>
			Library ({libraryTmdbIds.length})
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

	<div class="grid min-h-0 flex-1 grid-cols-3 grid-rows-1 gap-4">
		<!-- Col 1: Queue tasks -->
		<div class="flex min-h-0 flex-col gap-2 overflow-y-auto">
			{#if recTasks.length > 0}
				<div class="flex items-center justify-between text-xs text-base-content/50">
					<span>
						{pendingCount} pending, {runningCount} running, {completedCount} completed, {failedCount}
						failed
					</span>
					<div class="flex gap-1">
						<button class="btn btn-ghost btn-xs" onclick={() => queueService.clearCompleted()}>
							Clear Done
						</button>
						<button
							class="btn text-error btn-ghost btn-xs"
							onclick={async () => {
								await Promise.all(recTasks.map((t) => queueService.cancelTask(t.id)));
								await queueService.fetchTasks(undefined, 'recommendations:fetch');
							}}
						>
							Clear All
						</button>
					</div>
				</div>
			{/if}

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
								<span class="badge badge-ghost badge-xs">L{task.payload.level ?? 1}</span>
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
															<th>Type</th>
															<th>Level</th>
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
																<td>{rec.recommendedMediaType}</td>
																<td>{rec.level}</td>
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

		<!-- Col 2: Top Recommended table -->
		<div class="flex min-h-0 flex-col gap-2 overflow-y-auto">
			<h3 class="text-sm font-semibold">
				Top Recommended {label === 'Movie' ? 'Movies' : 'TV Shows'}
			</h3>
			<TopRecommendedTable
				bind:this={topTable}
				{mediaType}
				{selectedIndex}
				labelMap={labelEmojiMap}
				onrowclick={handleRowClick}
			/>
		</div>

		<!-- Col 3: Selected movie detail -->
		<div class="flex min-h-0 flex-col overflow-y-auto pr-1">
			{#if !selectedMovie}
				<p class="py-12 text-center text-sm text-base-content/50">
					Select a {label === 'Movie' ? 'movie' : 'TV show'} to see details
				</p>
			{:else if detailLoading && !selectedDetail}
				<div class="flex justify-center py-12">
					<span class="loading loading-lg loading-spinner"></span>
				</div>
			{:else}
				{@const data = selectedDetail?.data ?? null}
				<div class="flex flex-col gap-4">
					{#if backdropUrl(data)}
						<img src={backdropUrl(data)} alt="" class="h-48 w-full rounded-lg object-cover" />
					{/if}

					<!-- Sentiment Labels -->
					{#if labelDefs.length > 0 && wallet}
						<div class="rounded-lg bg-base-200 p-3">
							<h4 class="mb-2 text-sm font-semibold">Your Rating</h4>
							<RecommendationLabelGrid
								labels={labelDefs}
								{activeLabelId}
								loading={labelLoading}
								onlabelclick={handleLabelClick}
							/>
						</div>
					{/if}

					<div class="flex gap-4">
						{#if posterUrl(data)}
							<img
								src={posterUrl(data)}
								alt=""
								class="h-48 w-32 flex-shrink-0 rounded-lg object-cover shadow-md"
							/>
						{/if}
						<div class="min-w-0 flex-1">
							<h3 class="text-lg font-bold">
								{selectedMovie.title ?? '—'}
							</h3>
							{#if data?.original_title && data.original_title !== selectedMovie.title}
								<p class="text-sm text-base-content/50">{data.original_title}</p>
							{/if}
							{#if genres(data).length > 0}
								<div class="mt-1 flex flex-wrap gap-1">
									{#each genres(data) as genre}
										<span class="badge badge-outline badge-sm">{genre}</span>
									{/each}
								</div>
							{/if}

							<!-- Metadata grid -->
							<div class="mt-2 grid grid-cols-2 gap-x-4 gap-y-1 text-xs">
								{#if data?.release_date}
									<span class="text-base-content/50">Release Date</span>
									<span>{data.release_date}</span>
								{/if}
								{#if data?.vote_average != null}
									<span class="text-base-content/50">Rating</span>
									<span>{Number(data.vote_average).toFixed(1)} / 10</span>
								{/if}
								{#if data?.vote_count != null}
									<span class="text-base-content/50">Vote Count</span>
									<span>{Number(data.vote_count).toLocaleString()}</span>
								{/if}
								{#if data?.popularity != null}
									<span class="text-base-content/50">Popularity</span>
									<span>{Number(data.popularity).toFixed(1)}</span>
								{/if}
								{#if data?.original_language}
									<span class="text-base-content/50">Language</span>
									<span class="uppercase">{data.original_language}</span>
								{/if}
								{#if data?.runtime != null}
									<span class="text-base-content/50">Runtime</span>
									<span>{data.runtime} min</span>
								{/if}
								{#if data?.status}
									<span class="text-base-content/50">Status</span>
									<span>{data.status}</span>
								{/if}
								{#if data?.budget != null && Number(data.budget) > 0}
									<span class="text-base-content/50">Budget</span>
									<span>${Number(data.budget).toLocaleString()}</span>
								{/if}
								{#if data?.revenue != null && Number(data.revenue) > 0}
									<span class="text-base-content/50">Revenue</span>
									<span>${Number(data.revenue).toLocaleString()}</span>
								{/if}
								<span class="text-base-content/50">TMDB ID</span>
								<span>{selectedMovie.tmdbId}</span>
							</div>
						</div>
					</div>

					<!-- Recommendation stats -->
					<div class="rounded-lg bg-base-200 p-3">
						<h4 class="mb-2 text-sm font-semibold">Recommendation Stats</h4>
						<div class="grid grid-cols-2 gap-x-4 gap-y-1 text-sm">
							<span class="text-base-content/50">Times Recommended</span>
							<span>{selectedMovie.count}</span>
							<span class="text-base-content/50">Score</span>
							<span class="font-semibold">{selectedMovie.score}</span>
							{#if selectedDetail?.minLevel != null}
								<span class="text-base-content/50">Min Level</span>
								<span>{selectedDetail.minLevel}</span>
							{/if}
							{#each Object.entries(selectedMovie.levelCounts) as [lvl, cnt]}
								{#if cnt > 0}
									<span class="text-base-content/50">Level {lvl}</span>
									<span>
										{cnt}x ({selectedMovie.levelPercentages[lvl] ?? 0}%)
									</span>
								{/if}
							{/each}
						</div>
					</div>

					<!-- Overview -->
					{#if data?.overview}
						<div>
							<h4 class="mb-1 text-sm font-semibold">Overview</h4>
							<p class="text-sm leading-relaxed text-base-content/70">
								{data.overview}
							</p>
						</div>
					{/if}

					{#if data?.tagline}
						<p class="text-sm text-base-content/50 italic">"{data.tagline}"</p>
					{/if}

					<!-- Production companies -->
					{#if Array.isArray(data?.production_companies) && (data.production_companies as Array<{ name: string }>).length > 0}
						<div>
							<h4 class="mb-1 text-sm font-semibold">Production</h4>
							<p class="text-sm text-base-content/70">
								{(data.production_companies as Array<{ name: string }>)
									.map((c) => c.name)
									.join(', ')}
							</p>
						</div>
					{/if}

					<!-- Sources -->
					{#if selectedDetail && selectedDetail.sources.length > 0}
						<div>
							<h4 class="mb-1 text-sm font-semibold">
								Recommended From ({selectedDetail.sources.length})
							</h4>
							<div class="flex flex-wrap gap-1">
								{#each selectedDetail.sources as source}
									<span class="badge badge-ghost badge-sm">
										{source.title ?? `TMDB #${source.tmdbId}`}
									</span>
								{/each}
							</div>
						</div>
					{/if}
				</div>
			{/if}
		</div>
	</div>
</div>
