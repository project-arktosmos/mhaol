<script lang="ts">
	import { onMount } from 'svelte';
	import classNames from 'classnames';
	import type { DisplayTMDBMovie } from '$types/tmdb.type';
	import { tmdbService } from '$services/tmdb.service';
	import { tmdbAdapter } from '$adapters/classes/tmdb.adapter';

	type DiscoverCategory = 'now_playing' | 'popular' | 'upcoming' | 'top_rated';

	let discoverCategory = $state<DiscoverCategory>('popular');
	let movies = $state<DisplayTMDBMovie[]>([]);
	let loading = $state(false);
	let error = $state<string | null>(null);
	let page = $state(1);
	let totalPages = $state(0);

	// Search state
	let searchQuery = $state('');
	let searchActive = $state(false);

	// API key fallback
	let tmdbApiKey = $state('');
	let apiKeyFromEnv = $state(false);

	const categoryLabels: Record<DiscoverCategory, string> = {
		now_playing: 'Now Playing',
		popular: 'Popular',
		upcoming: 'Upcoming',
		top_rated: 'Top Rated'
	};

	onMount(() => {
		if (tmdbService.isConfigured()) {
			apiKeyFromEnv = true;
		} else {
			const savedKey = localStorage.getItem('tmdb_api_key');
			if (savedKey) {
				tmdbApiKey = savedKey;
				tmdbService.setApiKey(savedKey);
			}
		}
		loadMovies(discoverCategory, 1);
	});

	async function loadMovies(category: DiscoverCategory, p: number = 1) {
		if (!tmdbService.isConfigured()) {
			error = 'TMDB API key not configured';
			return;
		}

		loading = true;
		error = null;
		searchActive = false;

		try {
			let response;
			switch (category) {
				case 'now_playing':
					response = await tmdbService.getNowPlaying(p);
					break;
				case 'popular':
					response = await tmdbService.getPopular(p);
					break;
				case 'upcoming':
					response = await tmdbService.getUpcoming(p);
					break;
				case 'top_rated':
					response = await tmdbService.getTopRated(p);
					break;
			}

			if (response) {
				movies = tmdbAdapter.moviesToDisplay(response.results);
				page = response.page;
				totalPages = response.total_pages;
			}
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	}

	async function searchMovies(p: number = 1) {
		if (!searchQuery.trim()) {
			searchActive = false;
			loadMovies(discoverCategory, 1);
			return;
		}

		if (!tmdbService.isConfigured()) {
			error = 'TMDB API key not configured';
			return;
		}

		loading = true;
		error = null;
		searchActive = true;

		try {
			const response = await tmdbService.searchMovies(searchQuery.trim(), p);
			if (response) {
				movies = tmdbAdapter.moviesToDisplay(response.results);
				page = response.page;
				totalPages = response.total_pages;
			}
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
		loadMovies(category, 1);
	}

	function handlePageChange(newPage: number) {
		page = newPage;
		if (searchActive) {
			searchMovies(newPage);
		} else {
			loadMovies(discoverCategory, newPage);
		}
	}

	function handleSearchKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			page = 1;
			searchMovies(1);
		}
	}

	function saveApiKey() {
		if (tmdbApiKey.trim()) {
			localStorage.setItem('tmdb_api_key', tmdbApiKey.trim());
			tmdbService.setApiKey(tmdbApiKey.trim());
			loadMovies(discoverCategory, 1);
		}
	}
</script>

<div class="container mx-auto p-4">
	<div class="mb-6">
		<h1 class="text-3xl font-bold">Movies</h1>
		<p class="text-sm opacity-70">Search and discover movies from TMDB</p>
	</div>

	{#if !apiKeyFromEnv && !tmdbService.isConfigured()}
		<div class="collapse collapse-arrow mb-4 bg-base-200">
			<input type="checkbox" checked />
			<div class="collapse-title text-sm font-medium">
				TMDB API Key Configuration
				<span class="badge badge-warning badge-sm ml-2">Not Set</span>
			</div>
			<div class="collapse-content">
				<p class="mb-2 text-sm opacity-70">
					Get your free API key from
					<a
						href="https://www.themoviedb.org/settings/api"
						target="_blank"
						rel="noopener noreferrer"
						class="link link-primary"
					>
						TMDB
					</a>
				</p>
				<div class="flex gap-2">
					<input
						type="password"
						placeholder="Enter your TMDB API key"
						class="input input-bordered input-sm flex-1"
						bind:value={tmdbApiKey}
					/>
					<button class="btn btn-primary btn-sm" onclick={saveApiKey}>Save</button>
				</div>
			</div>
		</div>
	{/if}

	<!-- Search Bar -->
	<div class="mb-4">
		<input
			type="text"
			placeholder="Search movies..."
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
					loadMovies(discoverCategory, 1);
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
	{:else if movies.length === 0}
		<div class="rounded-lg bg-base-200 p-8 text-center">
			<p class="opacity-50">No movies found.</p>
		</div>
	{:else}
		<div class="mb-4 text-sm opacity-70">
			Page {page} of {totalPages}
		</div>

		<div class="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5">
			{#each movies as movie (movie.id)}
				<a
					href="/movies/{movie.id}"
					class="card bg-base-200 shadow-md transition-transform hover:scale-105"
				>
					<figure class="aspect-[2/3]">
						{#if movie.posterUrl}
							<img
								src={movie.posterUrl}
								alt={movie.title}
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
						<h3 class="card-title line-clamp-2 text-sm">{movie.title}</h3>
						<div class="flex items-center gap-2 text-xs opacity-70">
							<span>{movie.releaseYear}</span>
							{#if movie.voteAverage > 0}
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
									{movie.voteAverage.toFixed(1)}
								</span>
							{/if}
						</div>
						{#if movie.genres.length > 0}
							<div class="flex flex-wrap gap-1">
								{#each movie.genres.slice(0, 2) as genre}
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
