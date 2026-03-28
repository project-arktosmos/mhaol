<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import classNames from 'classnames';
	import { queueService } from 'ui-lib/services/queue.service';
	import { musicRecommendationsService } from 'ui-lib/services/music-recommendations.service';
	import { recommendationLabelsService } from 'ui-lib/services/recommendation-labels.service';
	import { profileService } from 'ui-lib/services/profile.service';
	import { fetchJson } from 'ui-lib/transport/fetch-helpers';
	import { getArtistImageUrl } from 'addons/musicbrainz/transform';
	import type { QueueTask } from 'ui-lib/types/queue.type';
	import type {
		MusicRecommendationRow,
		MusicRecommendationsStatus,
		TopRecommendedArtist,
		TopRecommendedArtistDetail
	} from 'ui-lib/types/music-recommendations.type';
	import type { RecommendationLabel } from 'ui-lib/types/recommendation-label.type';
	import { albumStrategy } from 'ui-lib/services/catalog-strategies/album.strategy';
	import { isAlbum } from 'ui-lib/types/catalog.type';
	import MusicTopRecommendedTable from './MusicTopRecommendedTable.svelte';
	import RecommendationLabelGrid from './RecommendationLabelGrid.svelte';

	interface Props {
		pinnedAlbumIds: string[];
		favoritedAlbumIds: string[];
	}

	let { pinnedAlbumIds, favoritedAlbumIds }: Props = $props();

	// === Resolved artist MBIDs from albums ===
	let pinnedArtistMbids = $state<string[]>([]);
	let favoritedArtistMbids = $state<string[]>([]);
	let resolvingArtists = $state(false);

	async function resolveArtistsFromAlbums(albumIds: string[]): Promise<string[]> {
		if (albumIds.length === 0) return [];
		if (!albumStrategy.resolveByIds) return [];
		const items = await albumStrategy.resolveByIds(albumIds);
		const mbids = new Set<string>();
		for (const item of items) {
			if (isAlbum(item)) {
				for (const author of item.metadata.authors) {
					if (author.source === 'musicbrainz' && author.role === 'artist') {
						mbids.add(author.id);
					}
				}
			}
		}
		return [...mbids];
	}

	// === Queue state ===
	let status = $state<MusicRecommendationsStatus | null>(null);
	let enqueueing = $state(false);
	let expandedTaskId = $state<string | null>(null);
	let expandedRecs = $state<Map<string, MusicRecommendationRow[]>>(new Map());
	let loadingRecs = $state<Set<string>>(new Set());

	let topTable: ReturnType<typeof MusicTopRecommendedTable>;

	const queueStore = queueService.store;

	let recTasks = $derived(
		$queueStore.tasks.filter((t) => t.taskType === 'music-recommendations:fetch')
	);
	let connected = $derived($queueStore.connected);

	let allMbids = $derived(() => {
		const ids = new Set<string>();
		for (const id of pinnedArtistMbids) ids.add(id);
		for (const id of favoritedArtistMbids) ids.add(id);
		return ids;
	});

	let pendingCount = $derived(recTasks.filter((t) => t.status === 'pending').length);
	let runningCount = $derived(recTasks.filter((t) => t.status === 'running').length);
	let completedCount = $derived(recTasks.filter((t) => t.status === 'completed').length);
	let failedCount = $derived(recTasks.filter((t) => t.status === 'failed').length);

	// === Explore state ===
	let detailMap = $state<Map<string, TopRecommendedArtistDetail>>(new Map());
	let detailLoading = $state(false);
	let selectedIndex = $state<number | null>(null);
	let selectedArtist = $state<TopRecommendedArtist | null>(null);

	let labelDefs = $state<RecommendationLabel[]>([]);
	let assignmentMap = $state<Map<string, string>>(new Map());
	let labelLoading = $state(false);
	let wallet = $state('');

	// Discography for selected artist
	let discography = $state<
		Array<{ id: string; title: string; year: string | null; coverUrl: string | null }>
	>([]);
	let discographyLoading = $state(false);

	const profileStore = profileService.state;
	profileStore.subscribe((s) => {
		wallet = s.local.wallet;
	});

	let activeLabelId = $derived.by(() => {
		if (!selectedArtist) return null;
		const key = `${selectedArtist.mbid}:${selectedArtist.type}`;
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
		selectedArtist ? (detailMap.get(selectedArtist.mbid) ?? null) : null
	);

	// === Queue functions ===
	async function loadStatus() {
		try {
			status = await musicRecommendationsService.getStatus();
		} catch {
			/* best-effort */
		}
	}

	async function enqueueAndRefresh(mbids: string[]) {
		if (mbids.length === 0) return;
		enqueueing = true;
		try {
			await musicRecommendationsService.bulkEnqueue(mbids.map((mbid) => ({ mbid })));
			await Promise.all([
				loadStatus(),
				queueService.fetchTasks(undefined, 'music-recommendations:fetch')
			]);
		} catch {
			/* best-effort */
		} finally {
			enqueueing = false;
		}
	}

	function enqueueAll() {
		enqueueAndRefresh([...allMbids()]);
	}

	function enqueuePinned() {
		enqueueAndRefresh(pinnedArtistMbids);
	}

	function enqueueFavorited() {
		enqueueAndRefresh(favoritedArtistMbids);
	}

	async function toggleExpand(task: QueueTask) {
		const key = task.id;
		if (expandedTaskId === key) {
			expandedTaskId = null;
			return;
		}
		expandedTaskId = key;
		if (task.status !== 'completed') return;

		const mbid = task.payload.mbid as string;
		if (!mbid || expandedRecs.has(key)) return;

		loadingRecs = new Set([...loadingRecs, key]);
		try {
			const recs = await musicRecommendationsService.getForSource(mbid);
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
		const mbid = task.payload.mbid as string;
		const result = task.result as Record<string, unknown> | null;
		if (result?.name) return `${result.name}`;
		if (mbid) return mbid.substring(0, 8) + '...';
		return 'Unknown';
	}

	function formatTime(iso: string): string {
		return new Date(iso).toLocaleTimeString();
	}

	// === Explore functions ===
	async function loadDetails() {
		detailLoading = true;
		try {
			const artists = await musicRecommendationsService.getTopDetail();
			detailMap = new Map(artists.map((a) => [a.mbid, a]));
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
			const assignments = await musicRecommendationsService.getLabelAssignments(wallet);
			assignmentMap = new Map(
				assignments.map((a) => [`${a.recommendedMbid}:${a.recommendedType}`, a.labelId])
			);
		} catch {
			/* best-effort */
		}
	}

	async function handleLabelClick(labelId: string) {
		if (!selectedArtist || !wallet) return;
		const key = `${selectedArtist.mbid}:${selectedArtist.type}`;
		labelLoading = true;
		try {
			if (activeLabelId === labelId) {
				await musicRecommendationsService.removeLabel(
					wallet,
					selectedArtist.mbid,
					selectedArtist.type
				);
				const next = new Map(assignmentMap);
				next.delete(key);
				assignmentMap = next;
			} else {
				await musicRecommendationsService.setLabel(
					wallet,
					selectedArtist.mbid,
					selectedArtist.type,
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

	async function loadDiscography(mbid: string) {
		discographyLoading = true;
		discography = [];
		try {
			const data = await fetchJson<Record<string, unknown>>(`/api/musicbrainz/artist/${mbid}`);
			const rgs = (data['release-groups'] ?? data.releaseGroups) as
				| Array<Record<string, unknown>>
				| undefined;
			if (Array.isArray(rgs)) {
				discography = rgs
					.filter((rg) => rg['primary-type'] === 'Album' || rg['primaryType'] === 'Album')
					.map((rg) => ({
						id: rg.id as string,
						title: (rg.title as string) ?? '—',
						year: (rg['first-release-date'] as string)?.substring(0, 4) ?? null,
						coverUrl: `/api/musicbrainz/cover/${rg.id}/250`
					}))
					.sort((a, b) => (b.year ?? '').localeCompare(a.year ?? ''));
			}
		} catch {
			/* best-effort */
		} finally {
			discographyLoading = false;
		}
	}

	function handleRowClick(index: number, artist: TopRecommendedArtist) {
		selectedIndex = index;
		selectedArtist = artist;
		loadDiscography(artist.mbid);
	}

	// === Lifecycle ===
	onMount(async () => {
		loadStatus();
		topTable?.refresh();
		loadDetails();
		loadLabelDefs();
		loadAssignments();
		queueService.fetchTasks(undefined, 'music-recommendations:fetch');
		queueService.subscribe();
		// Resolve artist MBIDs from pinned/favorited albums
		resolvingArtists = true;
		try {
			const [pinned, favorited] = await Promise.all([
				resolveArtistsFromAlbums(pinnedAlbumIds),
				resolveArtistsFromAlbums(favoritedAlbumIds)
			]);
			pinnedArtistMbids = pinned;
			favoritedArtistMbids = favorited;
		} catch { /* best-effort */ }
		resolvingArtists = false;
	});

	onDestroy(() => {
		queueService.unsubscribe();
	});
</script>

<div class="flex max-h-[80vh] flex-col gap-4 overflow-hidden">
	<div class="flex items-center justify-between">
		<h2 class="text-lg font-bold">Music Recommendations</h2>
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
			disabled={enqueueing || resolvingArtists || allMbids().size === 0}
			onclick={enqueueAll}
		>
			{#if enqueueing || resolvingArtists}
				<span class="loading loading-xs loading-spinner"></span>
			{/if}
			Enqueue All ({allMbids().size})
		</button>
		<button
			class="btn btn-outline btn-sm"
			disabled={enqueueing || resolvingArtists || pinnedArtistMbids.length === 0}
			onclick={enqueuePinned}
		>
			Pinned ({pinnedArtistMbids.length})
		</button>
		<button
			class="btn btn-outline btn-sm"
			disabled={enqueueing || resolvingArtists || favoritedArtistMbids.length === 0}
			onclick={enqueueFavorited}
		>
			Favorites ({favoritedArtistMbids.length})
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
								await queueService.fetchTasks(undefined, 'music-recommendations:fetch');
							}}
						>
							Clear All
						</button>
					</div>
				</div>
			{/if}

			{#if recTasks.length === 0}
				<p class="py-8 text-center text-sm text-base-content/50">
					No recommendation tasks. Enqueue artists above to get started.
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
										<span class="badge badge-ghost badge-xs">{count} similar</span>
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
															<th>Artist</th>
															<th>Score</th>
															<th>Level</th>
														</tr>
													</thead>
													<tbody>
														{#each recs as rec (rec.id)}
															<tr>
																<td class="max-w-40 truncate">{rec.name ?? '—'}</td>
																<td>{rec.score != null ? Math.round(rec.score) : '—'}</td>
																<td>{rec.level}</td>
															</tr>
														{/each}
													</tbody>
												</table>
											</div>
										{:else if recs}
											<p class="text-xs text-base-content/50">
												No similar artists found for this source.
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

		<!-- Col 2: Top Recommended Artists table -->
		<div class="flex min-h-0 flex-col gap-2 overflow-y-auto">
			<h3 class="text-sm font-semibold">Top Recommended Artists</h3>
			<MusicTopRecommendedTable
				bind:this={topTable}
				{selectedIndex}
				labelMap={labelEmojiMap}
				onrowclick={handleRowClick}
			/>
		</div>

		<!-- Col 3: Selected artist detail -->
		<div class="flex min-h-0 flex-col overflow-y-auto pr-1">
			{#if !selectedArtist}
				<p class="py-12 text-center text-sm text-base-content/50">
					Select an artist to see details
				</p>
			{:else if detailLoading && !selectedDetail}
				<div class="flex justify-center py-12">
					<span class="loading loading-lg loading-spinner"></span>
				</div>
			{:else}
				{@const data = selectedDetail?.data ?? null}
				{@const imgUrl = getArtistImageUrl(selectedArtist.mbid, 500)}
				<div class="flex flex-col gap-4">
					{#if imgUrl}
						<img src={imgUrl} alt="" class="h-48 w-full rounded-lg object-cover" />
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
						<h3 class="text-lg font-bold">{selectedArtist.name ?? '—'}</h3>
						{#if data?.type}
							<span class="badge badge-outline badge-sm">{data.type}</span>
						{/if}
						{#if data?.comment}
							<p class="text-sm text-base-content/50">{data.comment}</p>
						{/if}
						{#if data?.gender}
							<span class="ml-1 text-xs text-base-content/40">{data.gender}</span>
						{/if}
					</div>

					<!-- Recommendation stats -->
					<div class="rounded-lg bg-base-200 p-3">
						<h4 class="mb-2 text-sm font-semibold">Recommendation Stats</h4>
						<div class="grid grid-cols-2 gap-x-4 gap-y-1 text-sm">
							<span class="text-base-content/50">Times Recommended</span>
							<span>{selectedArtist.count}</span>
							<span class="text-base-content/50">Score</span>
							<span class="font-semibold">{selectedArtist.score}</span>
							{#if selectedDetail?.minLevel != null}
								<span class="text-base-content/50">Min Level</span>
								<span>{selectedDetail.minLevel}</span>
							{/if}
							{#each Object.entries(selectedArtist.levelCounts) as [lvl, cnt]}
								{#if cnt > 0}
									<span class="text-base-content/50">Level {lvl}</span>
									<span>
										{cnt}x ({selectedArtist.levelPercentages[lvl] ?? 0}%)
									</span>
								{/if}
							{/each}
						</div>
					</div>

					<!-- Discography -->
					<div>
						<h4 class="mb-2 text-sm font-semibold">
							Discography
							{#if discographyLoading}
								<span class="loading ml-1 loading-xs loading-spinner"></span>
							{/if}
						</h4>
						{#if discography.length > 0}
							<div class="grid grid-cols-3 gap-2">
								{#each discography as album (album.id)}
									<a
										href="/media/music/{album.id}"
										class="flex flex-col gap-1 rounded-lg bg-base-200 p-2 transition-colors hover:bg-base-300"
									>
										{#if album.coverUrl}
											<img
												src={album.coverUrl}
												alt=""
												class="aspect-square w-full rounded object-cover"
											/>
										{:else}
											<div
												class="flex aspect-square w-full items-center justify-center rounded bg-base-300 text-xs text-base-content/30"
											>
												No art
											</div>
										{/if}
										<span class="truncate text-xs font-medium">{album.title}</span>
										{#if album.year}
											<span class="text-xs text-base-content/40">{album.year}</span>
										{/if}
									</a>
								{/each}
							</div>
						{:else if !discographyLoading}
							<p class="text-xs text-base-content/50">No albums found.</p>
						{/if}
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
										{source.name ?? source.mbid.substring(0, 8)}
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
