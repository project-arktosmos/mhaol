<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import classNames from 'classnames';
	import { queueService } from 'ui-lib/services/queue.service';
	import { bookRecommendationsService } from 'ui-lib/services/book-recommendations.service';
	import { recommendationLabelsService } from 'ui-lib/services/recommendation-labels.service';
	import { profileService } from 'ui-lib/services/profile.service';
	import type { QueueTask } from 'ui-lib/types/queue.type';
	import type {
		BookRecommendationRow,
		BookRecommendationsStatus,
		TopRecommendedBook,
		TopRecommendedBookDetail
	} from 'ui-lib/types/book-recommendations.type';
	import type { RecommendationLabel } from 'ui-lib/types/recommendation-label.type';
	import BookTopRecommendedTable from './BookTopRecommendedTable.svelte';
	import RecommendationLabelGrid from './RecommendationLabelGrid.svelte';

	interface Props {
		pinnedBookKeys: string[];
		favoritedBookKeys: string[];
	}

	let { pinnedBookKeys, favoritedBookKeys }: Props = $props();

	// === Queue state ===
	let status = $state<BookRecommendationsStatus | null>(null);
	let enqueueing = $state(false);
	let expandedTaskId = $state<string | null>(null);
	let expandedRecs = $state<Map<string, BookRecommendationRow[]>>(new Map());
	let loadingRecs = $state<Set<string>>(new Set());

	let topTable: ReturnType<typeof BookTopRecommendedTable>;

	const queueStore = queueService.store;

	let recTasks = $derived(
		$queueStore.tasks.filter((t) => t.taskType === 'book-recommendations:fetch')
	);
	let connected = $derived($queueStore.connected);

	let allKeys = $derived(() => {
		const ids = new Set<string>();
		for (const id of pinnedBookKeys) ids.add(id);
		for (const id of favoritedBookKeys) ids.add(id);
		return ids;
	});

	let pendingCount = $derived(recTasks.filter((t) => t.status === 'pending').length);
	let runningCount = $derived(recTasks.filter((t) => t.status === 'running').length);
	let completedCount = $derived(recTasks.filter((t) => t.status === 'completed').length);
	let failedCount = $derived(recTasks.filter((t) => t.status === 'failed').length);

	// === Explore state ===
	let detailMap = $state<Map<string, TopRecommendedBookDetail>>(new Map());
	let detailLoading = $state(false);
	let selectedIndex = $state<number | null>(null);
	let selectedBook = $state<TopRecommendedBook | null>(null);

	let labelDefs = $state<RecommendationLabel[]>([]);
	let assignmentMap = $state<Map<string, string>>(new Map());
	let labelLoading = $state(false);
	let wallet = $state('');

	const profileStore = profileService.state;
	profileStore.subscribe((s) => {
		wallet = s.local.wallet;
	});

	let activeLabelId = $derived.by(() => {
		if (!selectedBook) return null;
		return assignmentMap.get(selectedBook.key) ?? null;
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
		selectedBook ? (detailMap.get(selectedBook.key) ?? null) : null
	);

	// === Queue functions ===
	async function loadStatus() {
		try {
			status = await bookRecommendationsService.getStatus();
		} catch {
			/* best-effort */
		}
	}

	async function enqueueAndRefresh(keys: string[]) {
		if (keys.length === 0) return;
		enqueueing = true;
		try {
			await bookRecommendationsService.bulkEnqueue(keys.map((key) => ({ key })));
			await Promise.all([
				loadStatus(),
				queueService.fetchTasks(undefined, 'book-recommendations:fetch')
			]);
		} catch {
			/* best-effort */
		} finally {
			enqueueing = false;
		}
	}

	function enqueueAll() {
		enqueueAndRefresh([...allKeys()]);
	}

	function enqueuePinned() {
		enqueueAndRefresh(pinnedBookKeys);
	}

	function enqueueFavorited() {
		enqueueAndRefresh(favoritedBookKeys);
	}

	async function toggleExpand(task: QueueTask) {
		const id = task.id;
		if (expandedTaskId === id) {
			expandedTaskId = null;
			return;
		}
		expandedTaskId = id;
		if (task.status !== 'completed') return;

		const key = task.payload.key as string;
		if (!key || expandedRecs.has(id)) return;

		loadingRecs = new Set([...loadingRecs, id]);
		try {
			const recs = await bookRecommendationsService.getForSource(key);
			expandedRecs = new Map([...expandedRecs, [id, recs]]);
		} catch {
			/* best-effort */
		} finally {
			const next = new Set(loadingRecs);
			next.delete(id);
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
		const result = task.result as Record<string, unknown> | null;
		if (result?.title) return `${result.title}`;
		const key = task.payload.key as string;
		if (key) return key;
		return 'Unknown';
	}

	function formatTime(iso: string): string {
		return new Date(iso).toLocaleTimeString();
	}

	// === Explore functions ===
	async function loadDetails() {
		detailLoading = true;
		try {
			const books = await bookRecommendationsService.getTopDetail();
			detailMap = new Map(books.map((b) => [b.key, b]));
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
			const assignments = await bookRecommendationsService.getLabelAssignments(wallet);
			assignmentMap = new Map(assignments.map((a) => [a.recommendedKey, a.labelId]));
		} catch {
			/* best-effort */
		}
	}

	async function handleLabelClick(labelId: string) {
		if (!selectedBook || !wallet) return;
		labelLoading = true;
		try {
			if (activeLabelId === labelId) {
				await bookRecommendationsService.removeLabel(wallet, selectedBook.key);
				const next = new Map(assignmentMap);
				next.delete(selectedBook.key);
				assignmentMap = next;
			} else {
				await bookRecommendationsService.setLabel(wallet, selectedBook.key, labelId);
				const next = new Map(assignmentMap);
				next.set(selectedBook.key, labelId);
				assignmentMap = next;
			}
		} catch {
			/* best-effort */
		} finally {
			labelLoading = false;
		}
	}

	function handleRowClick(index: number, book: TopRecommendedBook) {
		selectedIndex = index;
		selectedBook = book;
	}

	// === Lifecycle ===
	onMount(() => {
		loadStatus();
		topTable?.refresh();
		loadDetails();
		loadLabelDefs();
		loadAssignments();
		queueService.fetchTasks(undefined, 'book-recommendations:fetch');
		queueService.subscribe();
	});

	onDestroy(() => {
		queueService.unsubscribe();
	});
</script>

<div class="flex max-h-[80vh] flex-col gap-4 overflow-hidden">
	<div class="flex items-center justify-between">
		<h2 class="text-lg font-bold">Book Recommendations</h2>
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
			disabled={enqueueing || allKeys().size === 0}
			onclick={enqueueAll}
		>
			{#if enqueueing}
				<span class="loading loading-xs loading-spinner"></span>
			{/if}
			Enqueue All ({allKeys().size})
		</button>
		<button
			class="btn btn-outline btn-sm"
			disabled={enqueueing || pinnedBookKeys.length === 0}
			onclick={enqueuePinned}
		>
			Pinned ({pinnedBookKeys.length})
		</button>
		<button
			class="btn btn-outline btn-sm"
			disabled={enqueueing || favoritedBookKeys.length === 0}
			onclick={enqueueFavorited}
		>
			Favorites ({favoritedBookKeys.length})
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
								await queueService.fetchTasks(undefined, 'book-recommendations:fetch');
							}}
						>
							Clear All
						</button>
					</div>
				</div>
			{/if}

			{#if recTasks.length === 0}
				<p class="py-8 text-center text-sm text-base-content/50">
					No recommendation tasks. Enqueue books above to get started.
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
										<span class="badge badge-ghost badge-xs">{count} related</span>
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
															<th>Book</th>
															<th>Score</th>
															<th>Level</th>
														</tr>
													</thead>
													<tbody>
														{#each recs as rec (rec.id)}
															<tr>
																<td class="max-w-40 truncate">{rec.title ?? '—'}</td>
																<td>{Math.round(rec.score)}</td>
																<td>{rec.level}</td>
															</tr>
														{/each}
													</tbody>
												</table>
											</div>
										{:else if recs}
											<p class="text-xs text-base-content/50">
												No related books found for this source.
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

		<!-- Col 2: Top Recommended Books table -->
		<div class="flex min-h-0 flex-col gap-2 overflow-y-auto">
			<h3 class="text-sm font-semibold">Top Recommended Books</h3>
			<BookTopRecommendedTable
				bind:this={topTable}
				{selectedIndex}
				labelMap={labelEmojiMap}
				onrowclick={handleRowClick}
			/>
		</div>

		<!-- Col 3: Selected book detail -->
		<div class="flex min-h-0 flex-col overflow-y-auto pr-1">
			{#if !selectedBook}
				<p class="py-12 text-center text-sm text-base-content/50">
					Select a book to see details
				</p>
			{:else if detailLoading && !selectedDetail}
				<div class="flex justify-center py-12">
					<span class="loading loading-lg loading-spinner"></span>
				</div>
			{:else}
				{@const data = selectedDetail?.data ?? null}
				{@const coverId = data?.cover_i ?? data?.coverId}
				<div class="flex flex-col gap-4">
					{#if coverId}
						<img
							src="/api/openlibrary/cover/{coverId}/L"
							alt=""
							class="h-48 w-full rounded-lg object-contain"
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

					<div class="min-w-0">
						<h3 class="text-lg font-bold">{selectedBook.title ?? '—'}</h3>
						{#if data?.authors}
							{@const authors = data.authors}
							{#if Array.isArray(authors)}
								<p class="text-sm text-base-content/60">
									{authors.map((a) => (typeof a === 'object' ? a.name : a)).join(', ')}
								</p>
							{/if}
						{/if}
						{#if data?.first_publish_year ?? data?.firstPublishYear}
							<span class="badge badge-outline badge-sm">
								{data.first_publish_year ?? data.firstPublishYear}
							</span>
						{/if}
					</div>

					<!-- Subjects -->
					{#if data?.subject}
						{@const subjects = Array.isArray(data.subject) ? data.subject : []}
						{#if subjects.length > 0}
							<div>
								<h4 class="mb-1 text-sm font-semibold">Subjects</h4>
								<div class="flex flex-wrap gap-1">
									{#each subjects.slice(0, 15) as subj}
										<span class="badge badge-ghost badge-sm">{subj}</span>
									{/each}
									{#if subjects.length > 15}
										<span class="badge badge-ghost badge-sm">+{subjects.length - 15}</span>
									{/if}
								</div>
							</div>
						{/if}
					{/if}

					<!-- Recommendation stats -->
					<div class="rounded-lg bg-base-200 p-3">
						<h4 class="mb-2 text-sm font-semibold">Recommendation Stats</h4>
						<div class="grid grid-cols-2 gap-x-4 gap-y-1 text-sm">
							<span class="text-base-content/50">Times Recommended</span>
							<span>{selectedBook.count}</span>
							<span class="text-base-content/50">Score</span>
							<span class="font-semibold">{selectedBook.score}</span>
							{#if selectedDetail?.minLevel != null}
								<span class="text-base-content/50">Min Level</span>
								<span>{selectedDetail.minLevel}</span>
							{/if}
							{#each Object.entries(selectedBook.levelCounts) as [lvl, cnt]}
								{#if cnt > 0}
									<span class="text-base-content/50">Level {lvl}</span>
									<span>
										{cnt}x ({selectedBook.levelPercentages[lvl] ?? 0}%)
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
										{source.title ?? source.key}
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
