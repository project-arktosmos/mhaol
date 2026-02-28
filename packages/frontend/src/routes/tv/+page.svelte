<script lang="ts">
	import { onMount } from 'svelte';
	import classNames from 'classnames';
	import type { DisplayTMDBTvShow } from 'tmdb/types';
	import { tvShowsToDisplay } from 'tmdb/transform';

	type DiscoverCategory = 'on_the_air' | 'popular' | 'airing_today' | 'top_rated';

	let discoverCategory = $state<DiscoverCategory>('popular');
	let shows = $state<DisplayTMDBTvShow[]>([]);
	let loading = $state(false);
	let error = $state<string | null>(null);
	let page = $state(1);
	let totalPages = $state(0);

	// Search state
	let searchQuery = $state('');
	let searchActive = $state(false);

	const categoryLabels: Record<DiscoverCategory, string> = {
		on_the_air: 'On The Air',
		popular: 'Popular',
		airing_today: 'Airing Today',
		top_rated: 'Top Rated'
	};

	onMount(() => {
		loadShows(discoverCategory, 1);
	});

	async function loadShows(category: DiscoverCategory, p: number = 1) {
		loading = true;
		error = null;
		searchActive = false;

		try {
			const res = await fetch(`/api/tmdb/tv?category=${category}&page=${p}`);
			const data = await res.json();
			if (!res.ok) {
				error = data.error ?? 'Failed to load TV shows';
				return;
			}
			shows = tvShowsToDisplay(data.results);
			page = data.page;
			totalPages = data.total_pages;
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	}

	async function searchShows(p: number = 1) {
		if (!searchQuery.trim()) {
			searchActive = false;
			loadShows(discoverCategory, 1);
			return;
		}

		loading = true;
		error = null;
		searchActive = true;

		try {
			const params = new URLSearchParams({ q: searchQuery.trim(), page: p.toString() });
			const res = await fetch(`/api/tmdb/search/tv?${params}`);
			const data = await res.json();
			if (!res.ok) {
				error = data.error ?? 'Search failed';
				return;
			}
			shows = tvShowsToDisplay(data.results);
			page = data.page;
			totalPages = data.total_pages;
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	}

	function handleCategoryChange(category: DiscoverCategory) {
		discoverCategory = category;
		searchQuery = '';
		searchActive = false;
		page = 1;
		loadShows(category, 1);
	}

	function handlePageChange(newPage: number) {
		page = newPage;
		if (searchActive) {
			searchShows(newPage);
		} else {
			loadShows(discoverCategory, newPage);
		}
	}

	function handleSearchKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			page = 1;
			searchShows(1);
		}
	}
</script>

<div class="container mx-auto p-4">
	<div class="mb-6">
		<h1 class="text-3xl font-bold">TV Shows</h1>
		<p class="text-sm opacity-70">Search and discover TV shows from TMDB</p>
	</div>

	<!-- Search Bar -->
	<div class="mb-4">
		<input
			type="text"
			placeholder="Search TV shows..."
			class="input input-bordered w-full"
			bind:value={searchQuery}
			onkeydown={handleSearchKeydown}
		/>
	</div>

	<!-- Category Tabs -->
	{#if !searchActive}
		<div class="mb-4 flex flex-wrap gap-2">
			{#each Object.entries(categoryLabels) as [key, label]}
				<button
					class={classNames('btn btn-sm', {
						'btn-primary': discoverCategory === key,
						'btn-ghost': discoverCategory !== key
					})}
					onclick={() => handleCategoryChange(key as DiscoverCategory)}
				>
					{label}
				</button>
			{/each}
		</div>
	{:else}
		<div class="mb-4 flex items-center gap-2">
			<span class="text-sm opacity-70">
				Search results for "{searchQuery}"
			</span>
			<button
				class="btn btn-ghost btn-xs"
				onclick={() => {
					searchQuery = '';
					searchActive = false;
					loadShows(discoverCategory, 1);
				}}
			>
				Clear
			</button>
		</div>
	{/if}

	{#if error}
		<div class="alert alert-error mb-4">
			<span>{error}</span>
			<button class="btn btn-ghost btn-sm" onclick={() => (error = null)}>x</button>
		</div>
	{/if}

	{#if loading}
		<div class="flex justify-center py-12">
			<span class="loading loading-spinner loading-lg"></span>
		</div>
	{:else if shows.length === 0}
		<div class="rounded-lg bg-base-200 p-8 text-center">
			<p class="opacity-50">No TV shows found.</p>
		</div>
	{:else}
		<div class="mb-4 text-sm opacity-70">
			Page {page} of {totalPages}
		</div>

		<div class="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5">
			{#each shows as show (show.id)}
				<a
					href="/tv/{show.id}"
					class="card bg-base-200 shadow-md transition-transform hover:scale-105"
				>
					<figure class="aspect-[2/3]">
						{#if show.posterUrl}
							<img
								src={show.posterUrl}
								alt={show.name}
								class="h-full w-full object-cover"
							/>
						{:else}
							<div
								class="flex h-full w-full items-center justify-center bg-base-300"
							>
								<span class="text-4xl opacity-30">?</span>
							</div>
						{/if}
					</figure>
					<div class="card-body p-3">
						<h3 class="card-title line-clamp-2 text-sm">{show.name}</h3>
						<div class="flex items-center gap-2 text-xs opacity-70">
							<span>{show.firstAirYear}</span>
							{#if show.voteAverage > 0}
								<span class="flex items-center gap-1">
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
									{show.voteAverage.toFixed(1)}
								</span>
							{/if}
						</div>
						{#if show.numberOfSeasons}
							<div class="text-xs opacity-50">
								{show.numberOfSeasons} season{show.numberOfSeasons !== 1
									? 's'
									: ''}
							</div>
						{/if}
						{#if show.genres.length > 0}
							<div class="flex flex-wrap gap-1">
								{#each show.genres.slice(0, 2) as genre}
									<span class="badge badge-ghost badge-xs">{genre}</span>
								{/each}
							</div>
						{/if}
					</div>
				</a>
			{/each}
		</div>

		{#if totalPages > 1}
			<div class="mt-6 flex justify-center gap-2">
				<button
					class="btn btn-sm"
					disabled={page <= 1}
					onclick={() => handlePageChange(page - 1)}
				>
					Previous
				</button>
				<span class="flex items-center px-4 text-sm">
					Page {page} of {totalPages}
				</span>
				<button
					class="btn btn-sm"
					disabled={page >= totalPages}
					onclick={() => handlePageChange(page + 1)}
				>
					Next
				</button>
			</div>
		{/if}
	{/if}
</div>
