<script lang="ts">
	import { page } from '$app/stores';
	import { base } from '$app/paths';
	import { profileService } from 'ui-lib/services/profile.service';
	import { identityAdapter } from 'ui-lib/adapters/classes/identity.adapter';
	import type { ProfileDetail } from 'ui-lib/types/profile.type';

	let detail: ProfileDetail | null = $state(null);
	let loading = $state(true);
	let error: string | null = $state(null);

	const SERVICE_LABELS: Record<string, string> = {
		tmdb: 'Movie',
		'tmdb-tv': 'TV Show',
		'musicbrainz-album': 'Album',
		retroachievements: 'Game',
		youtube: 'YouTube',
		iptv: 'IPTV'
	};

	const SERVICE_ROUTES: Record<string, string> = {
		tmdb: 'movies',
		'tmdb-tv': 'tv',
		'musicbrainz-album': 'music',
		retroachievements: 'videogames',
		youtube: 'youtube',
		iptv: 'iptv'
	};

	function getFavoriteUrl(service: string, serviceId: string): string | null {
		const route = SERVICE_ROUTES[service];
		if (!route) return null;
		return `${base}/${route}/${encodeURIComponent(serviceId)}`;
	}

	function getServiceLabel(service: string): string {
		return SERVICE_LABELS[service] ?? service;
	}

	$effect(() => {
		const wallet = $page.params.address;
		if (wallet) {
			loading = true;
			error = null;
			profileService
				.fetchProfile(wallet)
				.then((d) => {
					detail = d;
					loading = false;
				})
				.catch((err) => {
					error = err instanceof Error ? err.message : 'Failed to load profile';
					loading = false;
				});
		}
	});
</script>

<div class="container mx-auto p-4">
	<div class="mb-4">
		<a href="/profiles" class="btn btn-ghost btn-sm gap-1">
			<span>&larr;</span> All Profiles
		</a>
	</div>

	{#if loading}
		<div class="flex justify-center py-12">
			<span class="loading loading-lg loading-spinner"></span>
		</div>
	{:else if error}
		<div class="alert alert-error">
			<span>{error}</span>
		</div>
	{:else if detail}
		<div class="card bg-base-200 mb-6">
			<div class="card-body">
				<div class="flex items-center gap-4">
					{#if detail.profile.profile_picture_url}
						<img
							src={detail.profile.profile_picture_url}
							alt={detail.profile.username}
							class="h-14 w-14 shrink-0 rounded-full object-cover"
						/>
					{:else}
						<div class="flex h-14 w-14 shrink-0 items-center justify-center rounded-full bg-primary text-primary-content text-xl font-bold">
							{detail.profile.username.charAt(0).toUpperCase()}
						</div>
					{/if}
					<div class="min-w-0">
						<h1 class="text-2xl font-bold">{detail.profile.username}</h1>
						<code class="break-all text-xs opacity-50">
							{detail.profile.wallet}
						</code>
					</div>
				</div>
				<div class="mt-2 flex gap-2 text-sm opacity-70">
					<span class="badge badge-outline">{detail.profile.favoriteCount} favorite{detail.profile.favoriteCount === 1 ? '' : 's'}</span>
					<span class="badge badge-outline">joined {new Date(detail.profile.added_at).toLocaleDateString()}</span>
				</div>
			</div>
		</div>

		<div class="mb-4">
			<h2 class="text-xl font-bold">Favorites</h2>
		</div>

		{#if detail.favorites.length === 0}
			<div class="rounded-lg bg-base-200 p-8 text-center">
				<p class="opacity-50">No favorites yet.</p>
			</div>
		{:else}
			<div class="flex flex-col gap-2">
				{#each detail.favorites as fav (fav.id)}
					{@const url = getFavoriteUrl(fav.service, fav.service_id)}
					{#if url}
						<a href={url} class="card bg-base-200 transition-colors hover:bg-base-300">
							<div class="card-body flex-row items-center gap-3 p-4">
								<svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 shrink-0 text-red-500" viewBox="0 0 24 24" fill="currentColor" stroke="currentColor" stroke-width="2">
									<path stroke-linecap="round" stroke-linejoin="round" d="M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z" />
								</svg>
								<div class="min-w-0 flex-1">
									<span class="font-semibold">{fav.label}</span>
									<div class="flex gap-2 text-xs opacity-50">
										<span class="badge badge-ghost badge-xs">{getServiceLabel(fav.service)}</span>
									</div>
								</div>
								<span class="text-xs opacity-40">
									{new Date(fav.created_at).toLocaleDateString()}
								</span>
							</div>
						</a>
					{:else}
						<div class="card bg-base-200">
							<div class="card-body flex-row items-center gap-3 p-4">
								<svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 shrink-0 text-red-500" viewBox="0 0 24 24" fill="currentColor" stroke="currentColor" stroke-width="2">
									<path stroke-linecap="round" stroke-linejoin="round" d="M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z" />
								</svg>
								<div class="min-w-0 flex-1">
									<span class="font-semibold">{fav.label}</span>
									<div class="flex gap-2 text-xs opacity-50">
										<span class="badge badge-ghost badge-xs">{getServiceLabel(fav.service)}</span>
									</div>
								</div>
								<span class="text-xs opacity-40">
									{new Date(fav.created_at).toLocaleDateString()}
								</span>
							</div>
						</div>
					{/if}
				{/each}
			</div>
		{/if}
	{/if}
</div>
