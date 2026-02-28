<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import type { DisplayTMDBMovieDetails } from 'tmdb/types';
	import { movieDetailsToDisplay } from 'tmdb/transform';

	let movie = $state<DisplayTMDBMovieDetails | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);

	onMount(async () => {
		const id = Number($page.params.id);
		if (isNaN(id)) {
			error = 'Invalid movie ID';
			loading = false;
			return;
		}

		try {
			const res = await fetch(`/api/tmdb/movies/${id}`);
			const data = await res.json();
			if (!res.ok) {
				error = data.error ?? 'Movie not found';
				return;
			}
			movie = movieDetailsToDisplay(data);
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	});
</script>

<div class="container mx-auto p-4">
	<div class="mb-4">
		<a href="/movies" class="btn btn-ghost btn-sm gap-1">
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
			Back to Movies
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
	{:else if movie}
		<!-- Backdrop -->
		{#if movie.backdropUrl}
			<div class="relative mb-6 overflow-hidden rounded-xl">
				<img
					src={movie.backdropUrl}
					alt={movie.title}
					class="h-64 w-full object-cover sm:h-80 md:h-96"
				/>
				<div
					class="absolute inset-0 bg-gradient-to-t from-base-100 via-base-100/50 to-transparent"
				></div>
			</div>
		{/if}

		<!-- Movie Info -->
		<div class="flex flex-col gap-6 md:flex-row">
			<!-- Poster -->
			<div class="flex-shrink-0">
				{#if movie.posterUrl}
					<img
						src={movie.posterUrl}
						alt={movie.title}
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
				<h1 class="text-3xl font-bold md:text-4xl">{movie.title}</h1>

				{#if movie.tagline}
					<p class="mt-1 text-lg italic opacity-70">{movie.tagline}</p>
				{/if}

				<div class="mt-3 flex flex-wrap items-center gap-3">
					<span class="text-sm opacity-70">{movie.releaseYear}</span>
					{#if movie.runtime}
						<span class="badge badge-outline">{movie.runtime}</span>
					{/if}
					{#if movie.voteAverage > 0}
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
							<span class="font-semibold">{movie.voteAverage.toFixed(1)}</span>
							<span class="text-xs opacity-50">({movie.voteCount})</span>
						</span>
					{/if}
				</div>

				{#if movie.genres.length > 0}
					<div class="mt-3 flex flex-wrap gap-2">
						{#each movie.genres as genre}
							<span class="badge badge-primary badge-outline">{genre}</span>
						{/each}
					</div>
				{/if}

				{#if movie.director}
					<div class="mt-4">
						<span class="text-sm font-semibold">Director:</span>
						<span class="text-sm">{movie.director}</span>
					</div>
				{/if}

				{#if movie.overview}
					<div class="mt-4">
						<h2 class="mb-2 text-lg font-semibold">Overview</h2>
						<p class="leading-relaxed opacity-80">{movie.overview}</p>
					</div>
				{/if}

				<!-- Budget & Revenue -->
				{#if movie.budget || movie.revenue}
					<div class="mt-4 flex flex-wrap gap-6">
						{#if movie.budget}
							<div>
								<span class="text-sm font-semibold">Budget:</span>
								<span class="text-sm">{movie.budget}</span>
							</div>
						{/if}
						{#if movie.revenue}
							<div>
								<span class="text-sm font-semibold">Revenue:</span>
								<span class="text-sm">{movie.revenue}</span>
							</div>
						{/if}
					</div>
				{/if}

				<!-- IMDb Link -->
				{#if movie.imdbId}
					<div class="mt-4">
						<a
							href="https://www.imdb.com/title/{movie.imdbId}"
							target="_blank"
							rel="noopener noreferrer"
							class="btn btn-outline btn-sm"
						>
							View on IMDb
						</a>
					</div>
				{/if}
			</div>
		</div>

		<!-- Cast -->
		{#if movie.cast.length > 0}
			<div class="mt-8">
				<h2 class="mb-4 text-xl font-semibold">Cast</h2>
				<div
					class="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5"
				>
					{#each movie.cast as member (member.id)}
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
	{/if}
</div>
