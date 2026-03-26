<script lang="ts">
	import { onMount } from 'svelte';
	import { recommendationsService } from 'ui-lib/services/recommendations.service';
	import { recommendationLabelsService } from 'ui-lib/services/recommendation-labels.service';
	import { profileService } from 'ui-lib/services/profile.service';
	import type {
		TopRecommendedMovie,
		TopRecommendedMovieDetail
	} from 'ui-lib/types/recommendations.type';
	import type { RecommendationLabel } from 'ui-lib/types/recommendation-label.type';
	import { getPosterUrl, getBackdropUrl } from 'addons/tmdb/transform';
	import TopRecommendedTable from './TopRecommendedTable.svelte';
	import RecommendationLabelGrid from './RecommendationLabelGrid.svelte';

	let topTable: ReturnType<typeof TopRecommendedTable>;
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

	async function loadDetails() {
		detailLoading = true;
		try {
			const movies = await recommendationsService.getTopMoviesDetail();
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

	onMount(() => {
		topTable?.refresh();
		loadDetails();
		loadLabelDefs();
		loadAssignments();
	});
</script>

<div class="flex max-h-[80vh] flex-col gap-4 overflow-hidden">
	<div class="flex items-center justify-between">
		<h2 class="text-lg font-bold">Explore Recommendations</h2>
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
	</div>

	<div class="grid min-h-0 flex-1 grid-cols-2 grid-rows-1 gap-4">
		<!-- Left: Top Recommended Movies table -->
		<div class="min-h-0 flex flex-col gap-2 overflow-y-auto">
			<h3 class="text-sm font-semibold">Top Recommended Movies</h3>
			<TopRecommendedTable
				bind:this={topTable}
				{selectedIndex}
				labelMap={labelEmojiMap}
				onrowclick={handleRowClick}
			/>
		</div>

		<!-- Right: Selected movie detail -->
		<div class="min-h-0 flex flex-col overflow-y-auto pr-1">
			{#if !selectedMovie}
				<p class="py-12 text-center text-sm text-base-content/50">
					Select a movie to see details
				</p>
			{:else if detailLoading && !selectedDetail}
				<div class="flex justify-center py-12">
					<span class="loading loading-lg loading-spinner"></span>
				</div>
			{:else}
				{@const data = selectedDetail?.data ?? null}
				<div class="flex flex-col gap-4">
					{#if backdropUrl(data)}
						<img
							src={backdropUrl(data)}
							alt=""
							class="h-48 w-full rounded-lg object-cover"
						/>
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
								<div class="mt-2 flex flex-wrap gap-1">
									{#each genres(data) as genre}
										<span class="badge badge-outline badge-sm">{genre}</span>
									{/each}
								</div>
							{/if}
						</div>
					</div>

					<!-- Metadata grid -->
					<div class="grid grid-cols-2 gap-x-4 gap-y-2 text-sm">
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
						<p class="text-sm italic text-base-content/50">"{data.tagline}"</p>
					{/if}

					<!-- Production companies -->
					{#if Array.isArray(data?.production_companies) && (data.production_companies as Array<{name: string}>).length > 0}
						<div>
							<h4 class="mb-1 text-sm font-semibold">Production</h4>
							<p class="text-sm text-base-content/70">
								{(data.production_companies as Array<{name: string}>).map((c) => c.name).join(', ')}
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
