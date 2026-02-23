<script lang="ts">
	import { onMount } from 'svelte';
	import classNames from 'classnames';
	import { page } from '$app/stores';
	import type { DisplayTMDBTvShowDetails, DisplayTMDBSeasonDetails } from '$types/tmdb.type';
	import { tmdbService } from '$services/tmdb.service';
	import { tmdbAdapter } from '$adapters/classes/tmdb.adapter';

	let show = $state<DisplayTMDBTvShowDetails | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);

	// Season expansion state
	let expandedSeasons = $state<Set<number>>(new Set());
	let seasonDetails = $state<Map<number, DisplayTMDBSeasonDetails>>(new Map());
	let loadingSeasons = $state<Set<number>>(new Set());

	onMount(async () => {
		const id = Number($page.params.id);
		if (isNaN(id)) {
			error = 'Invalid TV show ID';
			loading = false;
			return;
		}

		try {
			const data = await tmdbService.fetchTvShow(id);
			if (data) {
				show = tmdbAdapter.tvShowDetailsToDisplay(data);
			} else {
				error = 'TV show not found';
			}
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	});

	async function toggleSeason(seasonNumber: number) {
		const newExpanded = new Set(expandedSeasons);

		if (newExpanded.has(seasonNumber)) {
			newExpanded.delete(seasonNumber);
			expandedSeasons = newExpanded;
			return;
		}

		newExpanded.add(seasonNumber);
		expandedSeasons = newExpanded;

		// Load season details if not already loaded
		if (!seasonDetails.has(seasonNumber) && show) {
			const showId = Number($page.params.id);
			const newLoading = new Set(loadingSeasons);
			newLoading.add(seasonNumber);
			loadingSeasons = newLoading;

			try {
				const data = await tmdbService.fetchSeasonDetails(showId, seasonNumber);
				if (data) {
					const newDetails = new Map(seasonDetails);
					newDetails.set(seasonNumber, tmdbAdapter.seasonDetailsToDisplay(data));
					seasonDetails = newDetails;
				}
			} catch (e) {
				console.error(`Failed to load season ${seasonNumber}:`, e);
			} finally {
				const doneLoading = new Set(loadingSeasons);
				doneLoading.delete(seasonNumber);
				loadingSeasons = doneLoading;
			}
		}
	}
</script>

<div class="container mx-auto p-4">
	<div class="mb-4">
		<a href="/tv" class="btn btn-ghost btn-sm gap-1">
			<svg
				xmlns="http://www.w3.org/2000/svg"
				fill="none"
				viewBox="0 0 24 24"
				stroke-width="2"
				stroke="currentColor"
				class="h-4 w-4"
			>
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					d="M10.5 19.5L3 12m0 0l7.5-7.5M3 12h18"
				/>
			</svg>
			Back to TV Shows
		</a>
	</div>

	{#if loading}
		<div class="flex justify-center py-12">
			<span class="loading loading-spinner loading-lg"></span>
		</div>
	{:else if error}
		<div class="alert alert-error">
			<span>{error}</span>
		</div>
	{:else if show}
		<!-- Backdrop -->
		{#if show.backdropUrl}
			<div class="relative mb-6 overflow-hidden rounded-xl">
				<img
					src={show.backdropUrl}
					alt={show.name}
					class="h-64 w-full object-cover sm:h-80 md:h-96"
				/>
				<div
					class="absolute inset-0 bg-gradient-to-t from-base-100 via-base-100/50 to-transparent"
				></div>
			</div>
		{/if}

		<!-- Show Info -->
		<div class="flex flex-col gap-6 md:flex-row">
			<!-- Poster -->
			<div class="flex-shrink-0">
				{#if show.posterUrl}
					<img
						src={show.posterUrl}
						alt={show.name}
						class="mx-auto w-48 rounded-lg shadow-lg md:w-64"
					/>
				{:else}
					<div
						class="mx-auto flex h-72 w-48 items-center justify-center rounded-lg bg-base-300 md:h-96 md:w-64"
					>
						<span class="text-6xl opacity-30">?</span>
					</div>
				{/if}
			</div>

			<!-- Details -->
			<div class="flex-1">
				<h1 class="text-3xl font-bold md:text-4xl">{show.name}</h1>

				{#if show.tagline}
					<p class="mt-1 text-lg italic opacity-70">{show.tagline}</p>
				{/if}

				<div class="mt-3 flex flex-wrap items-center gap-3">
					<span class="text-sm opacity-70">
						{show.firstAirYear}{show.lastAirYear ? ` - ${show.lastAirYear}` : ''}
					</span>
					{#if show.status}
						<span class="badge badge-outline">{show.status}</span>
					{/if}
					{#if show.voteAverage > 0}
						<span class="flex items-center gap-1">
							<svg
								xmlns="http://www.w3.org/2000/svg"
								viewBox="0 0 24 24"
								fill="currentColor"
								class="h-5 w-5 text-yellow-500"
							>
								<path
									fill-rule="evenodd"
									d="M10.788 3.21c.448-1.077 1.976-1.077 2.424 0l2.082 5.007 5.404.433c1.164.093 1.636 1.545.749 2.305l-4.117 3.527 1.257 5.273c.271 1.136-.964 2.033-1.96 1.425L12 18.354 7.373 21.18c-.996.608-2.231-.29-1.96-1.425l1.257-5.273-4.117-3.527c-.887-.76-.415-2.212.749-2.305l5.404-.433 2.082-5.006z"
									clip-rule="evenodd"
								/>
							</svg>
							<span class="font-semibold">{show.voteAverage.toFixed(1)}</span>
							<span class="text-xs opacity-50">({show.voteCount})</span>
						</span>
					{/if}
				</div>

				{#if show.genres.length > 0}
					<div class="mt-3 flex flex-wrap gap-2">
						{#each show.genres as genre}
							<span class="badge badge-primary badge-outline">{genre}</span>
						{/each}
					</div>
				{/if}

				{#if show.networks.length > 0}
					<div class="mt-4">
						<span class="text-sm font-semibold">Networks:</span>
						<span class="text-sm">{show.networks.join(', ')}</span>
					</div>
				{/if}

				{#if show.createdBy.length > 0}
					<div class="mt-2">
						<span class="text-sm font-semibold">Created by:</span>
						<span class="text-sm">{show.createdBy.join(', ')}</span>
					</div>
				{/if}

				{#if show.numberOfSeasons || show.numberOfEpisodes}
					<div class="mt-2 flex gap-4 text-sm opacity-70">
						{#if show.numberOfSeasons}
							<span>
								{show.numberOfSeasons} season{show.numberOfSeasons !== 1
									? 's'
									: ''}
							</span>
						{/if}
						{#if show.numberOfEpisodes}
							<span>
								{show.numberOfEpisodes} episode{show.numberOfEpisodes !== 1
									? 's'
									: ''}
							</span>
						{/if}
					</div>
				{/if}

				{#if show.overview}
					<div class="mt-4">
						<h2 class="mb-2 text-lg font-semibold">Overview</h2>
						<p class="leading-relaxed opacity-80">{show.overview}</p>
					</div>
				{/if}
			</div>
		</div>

		<!-- Cast -->
		{#if show.cast.length > 0}
			<div class="mt-8">
				<h2 class="mb-4 text-xl font-semibold">Cast</h2>
				<div
					class="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5"
				>
					{#each show.cast as member (member.id)}
						<div class="flex items-center gap-3 rounded-lg bg-base-200 p-3">
							{#if member.profileUrl}
								<img
									src={member.profileUrl}
									alt={member.name}
									class="h-16 w-12 flex-shrink-0 rounded object-cover"
								/>
							{:else}
								<div
									class="flex h-16 w-12 flex-shrink-0 items-center justify-center rounded bg-base-300"
								>
									<span class="text-xs opacity-50">N/A</span>
								</div>
							{/if}
							<div class="min-w-0">
								<div class="truncate text-sm font-medium">{member.name}</div>
								<div class="truncate text-xs opacity-60">
									{member.character}
								</div>
							</div>
						</div>
					{/each}
				</div>
			</div>
		{/if}

		<!-- Seasons -->
		{#if show.seasons.length > 0}
			<div class="mt-8">
				<h2 class="mb-4 text-xl font-semibold">Seasons</h2>
				<div class="space-y-3">
					{#each show.seasons as season (season.id)}
						{@const isExpanded = expandedSeasons.has(season.seasonNumber)}
						{@const isLoading = loadingSeasons.has(season.seasonNumber)}
						{@const details = seasonDetails.get(season.seasonNumber)}

						<div class="card bg-base-200">
							<!-- Season Header -->
							<button
								class="card-body flex-row items-center gap-4 p-4 text-left transition-colors hover:bg-base-300"
								onclick={() => toggleSeason(season.seasonNumber)}
							>
								{#if season.posterUrl}
									<img
										src={season.posterUrl}
										alt={season.name}
										class="h-20 w-14 flex-shrink-0 rounded object-cover"
									/>
								{:else}
									<div
										class="flex h-20 w-14 flex-shrink-0 items-center justify-center rounded bg-base-300"
									>
										<span class="text-xs opacity-50">S{season.seasonNumber}</span>
									</div>
								{/if}

								<div class="min-w-0 flex-1">
									<h3 class="font-bold">{season.name}</h3>
									<div class="flex items-center gap-2 text-sm opacity-70">
										{#if season.airDate}
											<span>{season.airDate.split('-')[0]}</span>
										{/if}
										<span>
											{season.episodeCount} episode{season.episodeCount !== 1
												? 's'
												: ''}
										</span>
									</div>
									{#if season.overview}
										<p class="mt-1 line-clamp-2 text-sm opacity-60">
											{season.overview}
										</p>
									{/if}
								</div>

								<svg
									xmlns="http://www.w3.org/2000/svg"
									fill="none"
									viewBox="0 0 24 24"
									stroke-width="2"
									stroke="currentColor"
									class={classNames(
										'h-5 w-5 flex-shrink-0 transition-transform',
										{ 'rotate-180': isExpanded }
									)}
								>
									<path
										stroke-linecap="round"
										stroke-linejoin="round"
										d="m19.5 8.25-7.5 7.5-7.5-7.5"
									/>
								</svg>
							</button>

							<!-- Episodes -->
							{#if isExpanded}
								<div class="border-t border-base-300">
									{#if isLoading}
										<div class="flex justify-center py-6">
											<span class="loading loading-spinner loading-md"></span>
										</div>
									{:else if details}
										<div class="overflow-x-auto">
											<table class="table table-sm w-full">
												<thead>
													<tr>
														<th class="w-16">Ep</th>
														<th>Title</th>
														<th class="w-24">Air Date</th>
														<th class="w-20">Rating</th>
														<th class="w-20">Runtime</th>
													</tr>
												</thead>
												<tbody>
													{#each details.episodes as episode (episode.id)}
														<tr class="hover">
															<td>
																<span class="badge badge-ghost badge-sm">
																	{episode.episodeNumber}
																</span>
															</td>
															<td>
																<div class="flex items-center gap-3">
																	{#if episode.stillUrl}
																		<img
																			src={episode.stillUrl}
																			alt={episode.name}
																			class="hidden h-10 w-16 flex-shrink-0 rounded object-cover sm:block"
																		/>
																	{/if}
																	<div class="min-w-0">
																		<div
																			class="truncate font-medium"
																		>
																			{episode.name}
																		</div>
																		{#if episode.overview}
																			<div
																				class="line-clamp-1 text-xs opacity-50"
																			>
																				{episode.overview}
																			</div>
																		{/if}
																	</div>
																</div>
															</td>
															<td class="text-xs opacity-70">
																{episode.airDate || '-'}
															</td>
															<td>
																{#if episode.voteAverage > 0}
																	<span
																		class="flex items-center gap-1 text-xs"
																	>
																		<svg
																			xmlns="http://www.w3.org/2000/svg"
																			viewBox="0 0 24 24"
																			fill="currentColor"
																			class="h-3 w-3 text-yellow-500"
																		>
																			<path
																				fill-rule="evenodd"
																				d="M10.788 3.21c.448-1.077 1.976-1.077 2.424 0l2.082 5.007 5.404.433c1.164.093 1.636 1.545.749 2.305l-4.117 3.527 1.257 5.273c.271 1.136-.964 2.033-1.96 1.425L12 18.354 7.373 21.18c-.996.608-2.231-.29-1.96-1.425l1.257-5.273-4.117-3.527c-.887-.76-.415-2.212.749-2.305l5.404-.433 2.082-5.006z"
																				clip-rule="evenodd"
																			/>
																		</svg>
																		{episode.voteAverage.toFixed(
																			1
																		)}
																	</span>
																{:else}
																	<span class="text-xs opacity-50"
																		>-</span
																	>
																{/if}
															</td>
															<td class="text-xs opacity-70">
																{#if episode.runtime}
																	{episode.runtime}m
																{:else}
																	-
																{/if}
															</td>
														</tr>
													{/each}
												</tbody>
											</table>
										</div>
									{:else}
										<div class="p-4 text-center text-sm opacity-50">
											Failed to load episodes
										</div>
									{/if}
								</div>
							{/if}
						</div>
					{/each}
				</div>
			</div>
		{/if}
	{/if}
</div>
