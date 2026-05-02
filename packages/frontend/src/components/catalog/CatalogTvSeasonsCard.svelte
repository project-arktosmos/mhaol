<script lang="ts">
	import classNames from 'classnames';
	import { base } from '$app/paths';

	interface Props {
		tmdbTvId: string;
	}
	let { tmdbTvId }: Props = $props();

	interface Season {
		seasonNumber: number;
		name: string;
		airYear?: number | null;
		episodeCount?: number | null;
		posterUrl?: string | null;
		overview?: string | null;
	}

	interface Episode {
		episodeNumber: number;
		seasonNumber: number;
		name: string;
		overview?: string | null;
		airDate?: string | null;
		runtime?: number | null;
		stillUrl?: string | null;
		voteAverage?: number | null;
	}

	type LoadStatus = 'idle' | 'loading' | 'done' | 'error';

	let seasons = $state<Season[]>([]);
	let seasonsStatus = $state<LoadStatus>('idle');
	let seasonsError = $state<string | null>(null);

	let expanded = $state<Record<number, boolean>>({});
	let episodes = $state<Record<number, Episode[]>>({});
	let episodesStatus = $state<Record<number, LoadStatus>>({});
	let episodesError = $state<Record<number, string | null>>({});

	let loadedForId: string | null = null;

	$effect(() => {
		if (!tmdbTvId) return;
		if (loadedForId === tmdbTvId) return;
		loadedForId = tmdbTvId;
		seasons = [];
		expanded = {};
		episodes = {};
		episodesStatus = {};
		episodesError = {};
		void loadSeasons(tmdbTvId);
	});

	async function loadSeasons(id: string) {
		seasonsStatus = 'loading';
		seasonsError = null;
		try {
			const res = await fetch(`${base}/api/catalog/tmdb-tv/${encodeURIComponent(id)}/seasons`, {
				cache: 'no-store'
			});
			if (!res.ok) throw new Error(await readError(res));
			seasons = (await res.json()) as Season[];
			seasonsStatus = 'done';
		} catch (err) {
			seasonsError = err instanceof Error ? err.message : 'Unknown error';
			seasonsStatus = 'error';
		}
	}

	async function loadEpisodes(seasonNumber: number) {
		if (!tmdbTvId) return;
		if (episodesStatus[seasonNumber] === 'loading' || episodesStatus[seasonNumber] === 'done') {
			return;
		}
		episodesStatus = { ...episodesStatus, [seasonNumber]: 'loading' };
		episodesError = { ...episodesError, [seasonNumber]: null };
		try {
			const res = await fetch(
				`${base}/api/catalog/tmdb-tv/${encodeURIComponent(tmdbTvId)}/season/${seasonNumber}/episodes`,
				{ cache: 'no-store' }
			);
			if (!res.ok) throw new Error(await readError(res));
			const list = (await res.json()) as Episode[];
			episodes = { ...episodes, [seasonNumber]: list };
			episodesStatus = { ...episodesStatus, [seasonNumber]: 'done' };
		} catch (err) {
			episodesError = {
				...episodesError,
				[seasonNumber]: err instanceof Error ? err.message : 'Unknown error'
			};
			episodesStatus = { ...episodesStatus, [seasonNumber]: 'error' };
		}
	}

	async function readError(res: Response): Promise<string> {
		try {
			const body = await res.json();
			if (body && typeof body.error === 'string') return body.error;
		} catch {
			// ignore
		}
		return `HTTP ${res.status}`;
	}

	function toggle(seasonNumber: number) {
		const isOpen = !!expanded[seasonNumber];
		expanded = { ...expanded, [seasonNumber]: !isOpen };
		if (!isOpen) void loadEpisodes(seasonNumber);
	}

	function formatRuntime(min: number | null | undefined): string {
		if (!min || !Number.isFinite(min) || min <= 0) return '';
		if (min < 60) return `${min}m`;
		const h = Math.floor(min / 60);
		const m = min % 60;
		return m === 0 ? `${h}h` : `${h}h ${m}m`;
	}
</script>

<div class="card border border-base-content/10 bg-base-200 p-4">
	<div class="mb-2 flex items-center justify-between gap-2">
		<h2 class="text-sm font-semibold text-base-content/70 uppercase">
			Seasons{seasons.length > 0 ? ` (${seasons.length})` : ''}
		</h2>
	</div>

	{#if seasonsStatus === 'loading' && seasons.length === 0}
		<p class="text-sm text-base-content/60">Loading…</p>
	{:else if seasonsStatus === 'error'}
		<p class="text-sm text-error">{seasonsError ?? 'Failed'}</p>
	{:else if seasons.length === 0}
		<p class="text-sm text-base-content/60">No seasons found.</p>
	{:else}
		<ul class="flex flex-col gap-2">
			{#each seasons as season (season.seasonNumber)}
				{@const isOpen = !!expanded[season.seasonNumber]}
				{@const epList = episodes[season.seasonNumber] ?? []}
				{@const epStatus = episodesStatus[season.seasonNumber] ?? 'idle'}
				{@const epError = episodesError[season.seasonNumber] ?? null}
				<li class="rounded border border-base-content/10 bg-base-100">
					<button
						type="button"
						class="flex w-full items-center gap-3 p-2 text-left hover:bg-base-200"
						onclick={() => toggle(season.seasonNumber)}
						aria-expanded={isOpen}
					>
						{#if season.posterUrl}
							<img
								src={season.posterUrl}
								alt={season.name}
								loading="lazy"
								class="h-16 w-12 shrink-0 rounded object-cover"
							/>
						{:else}
							<div
								class="h-16 w-12 shrink-0 rounded bg-base-300 text-center text-xs leading-[4rem] text-base-content/40"
							>
								S{season.seasonNumber}
							</div>
						{/if}
						<div class="min-w-0 flex-1">
							<div class="flex flex-wrap items-baseline gap-2">
								<p class="font-medium">{season.name}</p>
								{#if season.airYear}
									<span class="text-xs text-base-content/60">{season.airYear}</span>
								{/if}
							</div>
							<p class="text-xs text-base-content/60">
								{season.episodeCount ?? 0} episode{season.episodeCount === 1 ? '' : 's'}
							</p>
							{#if season.overview}
								<p class="mt-1 line-clamp-2 text-xs text-base-content/70">{season.overview}</p>
							{/if}
						</div>
						<span
							class={classNames('shrink-0 text-base-content/60 transition-transform', {
								'rotate-90': isOpen
							})}
							aria-hidden="true"
						>
							›
						</span>
					</button>

					{#if isOpen}
						<div class="border-t border-base-content/10 p-2">
							{#if epStatus === 'loading' && epList.length === 0}
								<p class="text-xs text-base-content/60">Loading episodes…</p>
							{:else if epStatus === 'error'}
								<p class="text-xs text-error">{epError ?? 'Failed'}</p>
							{:else if epList.length === 0}
								<p class="text-xs text-base-content/60">No episodes found.</p>
							{:else}
								<ol class="flex flex-col gap-1">
									{#each epList as ep (ep.episodeNumber)}
										<li
											class="flex items-start gap-2 rounded border border-base-content/10 bg-base-200 p-2 text-xs"
										>
											{#if ep.stillUrl}
												<img
													src={ep.stillUrl}
													alt={ep.name}
													loading="lazy"
													class="h-12 w-20 shrink-0 rounded object-cover"
												/>
											{/if}
											<div class="min-w-0 flex-1">
												<div class="flex flex-wrap items-baseline gap-2">
													<span class="font-mono text-base-content/60">
														S{String(ep.seasonNumber).padStart(2, '0')}E{String(
															ep.episodeNumber
														).padStart(2, '0')}
													</span>
													<span class="font-medium">{ep.name}</span>
													{#if ep.airDate}
														<span class="text-base-content/60">{ep.airDate}</span>
													{/if}
													{#if ep.runtime}
														<span class="text-base-content/60">{formatRuntime(ep.runtime)}</span>
													{/if}
													{#if ep.voteAverage && ep.voteAverage > 0}
														<span class="badge badge-ghost badge-xs"
															>★ {ep.voteAverage.toFixed(1)}</span
														>
													{/if}
												</div>
												{#if ep.overview}
													<p class="mt-1 text-base-content/70">{ep.overview}</p>
												{/if}
											</div>
										</li>
									{/each}
								</ol>
							{/if}
						</div>
					{/if}
				</li>
			{/each}
		</ul>
	{/if}
</div>
