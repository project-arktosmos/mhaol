<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import classNames from 'classnames';
	import { queueService } from 'ui-lib/services/queue.service';
	import { gameRecommendationsService } from 'ui-lib/services/game-recommendations.service';
	import { recommendationLabelsService } from 'ui-lib/services/recommendation-labels.service';
	import { profileService } from 'ui-lib/services/profile.service';
	import { fetchJson } from 'ui-lib/transport/fetch-helpers';
	import { raImageUrl } from 'addons/retroachievements/transform';
	import type { QueueTask } from 'ui-lib/types/queue.type';
	import type {
		GameRecommendationRow,
		GameRecommendationsStatus,
		TopRecommendedGame,
		TopRecommendedGameDetail
	} from 'ui-lib/types/game-recommendations.type';
	import type { RecommendationLabel } from 'ui-lib/types/recommendation-label.type';
	import GameTopRecommendedTable from './GameTopRecommendedTable.svelte';
	import RecommendationLabelGrid from './RecommendationLabelGrid.svelte';

	interface Props {
		pinnedGameIds: number[];
		favoritedGameIds: number[];
	}

	let { pinnedGameIds, favoritedGameIds }: Props = $props();

	// === Queue state ===
	let status = $state<GameRecommendationsStatus | null>(null);
	let enqueueing = $state(false);
	let expandedTaskId = $state<string | null>(null);
	let expandedRecs = $state<Map<string, GameRecommendationRow[]>>(new Map());
	let loadingRecs = $state<Set<string>>(new Set());

	let topTable: ReturnType<typeof GameTopRecommendedTable>;

	const queueStore = queueService.store;

	let recTasks = $derived(
		$queueStore.tasks.filter((t) => t.taskType === 'game-recommendations:fetch')
	);
	let connected = $derived($queueStore.connected);

	let allGameIds = $derived(() => {
		const ids = new Set<number>();
		for (const id of pinnedGameIds) ids.add(id);
		for (const id of favoritedGameIds) ids.add(id);
		return ids;
	});

	let pendingCount = $derived(recTasks.filter((t) => t.status === 'pending').length);
	let runningCount = $derived(recTasks.filter((t) => t.status === 'running').length);
	let completedCount = $derived(recTasks.filter((t) => t.status === 'completed').length);
	let failedCount = $derived(recTasks.filter((t) => t.status === 'failed').length);

	// === Explore state ===
	let detailMap = $state<Map<number, TopRecommendedGameDetail>>(new Map());
	let detailLoading = $state(false);
	let selectedIndex = $state<number | null>(null);
	let selectedGame = $state<TopRecommendedGame | null>(null);

	let labelDefs = $state<RecommendationLabel[]>([]);
	let assignmentMap = $state<Map<string, string>>(new Map());
	let labelLoading = $state(false);
	let wallet = $state('');

	// Game detail from RA API
	let gameDetail = $state<Record<string, unknown> | null>(null);
	let gameDetailLoading = $state(false);

	const profileStore = profileService.state;
	profileStore.subscribe((s) => {
		wallet = s.local.wallet;
	});

	let activeLabelId = $derived.by(() => {
		if (!selectedGame) return null;
		return assignmentMap.get(String(selectedGame.gameId)) ?? null;
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

	let selectedDetail = $derived(selectedGame ? (detailMap.get(selectedGame.gameId) ?? null) : null);

	// === Queue functions ===
	async function loadStatus() {
		try {
			status = await gameRecommendationsService.getStatus();
		} catch {
			/* best-effort */
		}
	}

	async function enqueueAndRefresh(gameIds: number[]) {
		if (gameIds.length === 0) return;
		enqueueing = true;
		try {
			await gameRecommendationsService.bulkEnqueue(gameIds.map((gameId) => ({ gameId })));
			await Promise.all([
				loadStatus(),
				queueService.fetchTasks(undefined, 'game-recommendations:fetch')
			]);
		} catch {
			/* best-effort */
		} finally {
			enqueueing = false;
		}
	}

	function enqueueAll() {
		enqueueAndRefresh([...allGameIds()]);
	}

	function enqueuePinned() {
		enqueueAndRefresh(pinnedGameIds);
	}

	function enqueueFavorited() {
		enqueueAndRefresh(favoritedGameIds);
	}

	async function toggleExpand(task: QueueTask) {
		const key = task.id;
		if (expandedTaskId === key) {
			expandedTaskId = null;
			return;
		}
		expandedTaskId = key;
		if (task.status !== 'completed') return;

		const gameId = task.payload.gameId as number;
		if (!gameId || expandedRecs.has(key)) return;

		loadingRecs = new Set([...loadingRecs, key]);
		try {
			const recs = await gameRecommendationsService.getForSource(gameId);
			expandedRecs = new Map([...expandedRecs, [key, recs]]);
		} catch {
			/* best-effort */
		} finally {
			const next = new Set(loadingRecs);
			next.delete(key);
			loadingRecs = next;
		}
	}

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
		const id = task.payload.gameId as number;
		const result = task.result as Record<string, unknown> | null;
		if (result?.title) return `${result.title}`;
		return `Game #${id}`;
	}

	function formatTime(iso: string): string {
		return new Date(iso).toLocaleTimeString();
	}

	// === Explore functions ===
	async function loadDetails() {
		detailLoading = true;
		try {
			const games = await gameRecommendationsService.getTopDetail();
			detailMap = new Map(games.map((g) => [g.gameId, g]));
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
			const assignments = await gameRecommendationsService.getLabelAssignments(wallet);
			assignmentMap = new Map(assignments.map((a) => [String(a.recommendedGameId), a.labelId]));
		} catch {
			/* best-effort */
		}
	}

	async function handleLabelClick(labelId: string) {
		if (!selectedGame || !wallet) return;
		const key = String(selectedGame.gameId);
		labelLoading = true;
		try {
			if (activeLabelId === labelId) {
				await gameRecommendationsService.removeLabel(wallet, selectedGame.gameId);
				const next = new Map(assignmentMap);
				next.delete(key);
				assignmentMap = next;
			} else {
				await gameRecommendationsService.setLabel(wallet, selectedGame.gameId, labelId);
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

	async function loadGameDetail(gameId: number) {
		gameDetailLoading = true;
		gameDetail = null;
		try {
			gameDetail = await fetchJson<Record<string, unknown>>(
				`/api/retroachievements/games/${gameId}`
			);
		} catch {
			/* best-effort */
		} finally {
			gameDetailLoading = false;
		}
	}

	function handleRowClick(index: number, game: TopRecommendedGame) {
		selectedIndex = index;
		selectedGame = game;
		loadGameDetail(game.gameId);
	}

	// === Lifecycle ===
	onMount(() => {
		loadStatus();
		topTable?.refresh();
		loadDetails();
		loadLabelDefs();
		loadAssignments();
		queueService.fetchTasks(undefined, 'game-recommendations:fetch');
		queueService.subscribe();
	});

	onDestroy(() => {
		queueService.unsubscribe();
	});
</script>

<div class="flex max-h-[80vh] flex-col gap-4 overflow-hidden">
	<div class="flex items-center justify-between">
		<h2 class="text-lg font-bold">Game Recommendations</h2>
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
			disabled={enqueueing || allGameIds().size === 0}
			onclick={enqueueAll}
		>
			{#if enqueueing}
				<span class="loading loading-xs loading-spinner"></span>
			{/if}
			Enqueue All ({allGameIds().size})
		</button>
		<button
			class="btn btn-outline btn-sm"
			disabled={enqueueing || pinnedGameIds.length === 0}
			onclick={enqueuePinned}
		>
			Pinned ({pinnedGameIds.length})
		</button>
		<button
			class="btn btn-outline btn-sm"
			disabled={enqueueing || favoritedGameIds.length === 0}
			onclick={enqueueFavorited}
		>
			Favorites ({favoritedGameIds.length})
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
								await queueService.fetchTasks(undefined, 'game-recommendations:fetch');
							}}
						>
							Clear All
						</button>
					</div>
				</div>
			{/if}

			{#if recTasks.length === 0}
				<p class="py-8 text-center text-sm text-base-content/50">
					No recommendation tasks. Enqueue games above to get started.
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
															<th>Game</th>
															<th>Genre</th>
															<th>Console</th>
															<th>Score</th>
														</tr>
													</thead>
													<tbody>
														{#each recs.slice(0, 20) as rec (rec.id)}
															<tr>
																<td class="max-w-32 truncate">{rec.title ?? '—'}</td>
																<td class="max-w-20 truncate text-xs">{rec.genre ?? '—'}</td>
																<td class="text-xs">{rec.consoleName ?? '—'}</td>
																<td class="text-xs">{Math.round(rec.score)}</td>
															</tr>
														{/each}
													</tbody>
												</table>
											</div>
										{:else if recs}
											<p class="text-xs text-base-content/50">No recommendations found.</p>
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

		<!-- Col 2: Top Recommended Games table -->
		<div class="flex min-h-0 flex-col gap-2 overflow-y-auto">
			<h3 class="text-sm font-semibold">Top Recommended Games</h3>
			<GameTopRecommendedTable
				bind:this={topTable}
				{selectedIndex}
				labelMap={labelEmojiMap}
				onrowclick={handleRowClick}
			/>
		</div>

		<!-- Col 3: Selected game detail -->
		<div class="flex min-h-0 flex-col overflow-y-auto pr-1">
			{#if !selectedGame}
				<p class="py-12 text-center text-sm text-base-content/50">Select a game to see details</p>
			{:else if gameDetailLoading && !gameDetail}
				<div class="flex justify-center py-12">
					<span class="loading loading-lg loading-spinner"></span>
				</div>
			{:else}
				<div class="flex flex-col gap-4">
					<!-- Game images -->
					{#if gameDetail?.ImageBoxArt}
						<img
							src={raImageUrl(gameDetail.ImageBoxArt as string)}
							alt=""
							class="h-48 w-full rounded-lg bg-base-200 object-contain"
						/>
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
						{#if gameDetail?.ImageIcon}
							<img
								src={raImageUrl(gameDetail.ImageIcon as string)}
								alt=""
								class="h-16 w-16 flex-shrink-0 rounded"
							/>
						{/if}
						<div class="min-w-0 flex-1">
							<h3 class="text-lg font-bold">{selectedGame.title ?? '—'}</h3>
							{#if gameDetail?.ConsoleName}
								<span class="badge badge-outline badge-sm">{gameDetail.ConsoleName}</span>
							{/if}

							<div class="mt-2 grid grid-cols-2 gap-x-4 gap-y-1 text-xs">
								{#if gameDetail?.Genre}
									<span class="text-base-content/50">Genre</span>
									<span>{gameDetail.Genre}</span>
								{/if}
								{#if gameDetail?.Developer}
									<span class="text-base-content/50">Developer</span>
									<span>{gameDetail.Developer}</span>
								{/if}
								{#if gameDetail?.Publisher}
									<span class="text-base-content/50">Publisher</span>
									<span>{gameDetail.Publisher}</span>
								{/if}
								{#if gameDetail?.Released}
									<span class="text-base-content/50">Released</span>
									<span>{gameDetail.Released}</span>
								{/if}
								{#if gameDetail?.NumAchievements}
									<span class="text-base-content/50">Achievements</span>
									<span>{gameDetail.NumAchievements}</span>
								{/if}
								{#if gameDetail?.Points}
									<span class="text-base-content/50">Points</span>
									<span>{gameDetail.Points}</span>
								{/if}
								<span class="text-base-content/50">RA ID</span>
								<span>{selectedGame.gameId}</span>
							</div>
						</div>
					</div>

					<!-- Screenshots -->
					{#if gameDetail?.ImageTitle || gameDetail?.ImageIngame}
						<div class="flex gap-2">
							{#if gameDetail.ImageTitle}
								<img
									src={raImageUrl(gameDetail.ImageTitle as string)}
									alt="Title screen"
									class="h-24 flex-1 rounded bg-base-200 object-contain"
								/>
							{/if}
							{#if gameDetail.ImageIngame}
								<img
									src={raImageUrl(gameDetail.ImageIngame as string)}
									alt="In-game"
									class="h-24 flex-1 rounded bg-base-200 object-contain"
								/>
							{/if}
						</div>
					{/if}

					<!-- Recommendation stats -->
					<div class="rounded-lg bg-base-200 p-3">
						<h4 class="mb-2 text-sm font-semibold">Recommendation Stats</h4>
						<div class="grid grid-cols-2 gap-x-4 gap-y-1 text-sm">
							<span class="text-base-content/50">Times Recommended</span>
							<span>{selectedGame.count}</span>
							<span class="text-base-content/50">Score</span>
							<span class="font-semibold">{selectedGame.score}</span>
							{#each Object.entries(selectedGame.levelCounts) as [lvl, cnt]}
								{#if cnt > 0}
									<span class="text-base-content/50">Level {lvl}</span>
									<span>
										{cnt}x ({selectedGame.levelPercentages[lvl] ?? 0}%)
									</span>
								{/if}
							{/each}
						</div>
					</div>

					<!-- Sources -->
					{#if selectedDetail && selectedDetail.sources.length > 0}
						<div>
							<h4 class="mb-1 text-sm font-semibold">
								Recommended From ({selectedDetail.sources.length})
							</h4>
							<div class="flex flex-wrap gap-1">
								{#each selectedDetail.sources as source}
									<span class="badge badge-ghost badge-sm">
										{source.title ?? `Game #${source.gameId}`}
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
