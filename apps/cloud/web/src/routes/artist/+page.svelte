<script lang="ts">
	import { onMount } from 'svelte';
	import { base } from '$app/paths';
	import { artistsService, type Artist } from '$lib/artists.service';

	const artistsStore = artistsService.state;

	let query = $state('');

	onMount(() => {
		void artistsService.refresh();
	});

	function initials(name: string): string {
		return name
			.split(/\s+/)
			.filter((p) => p.length > 0)
			.map((p) => p[0]!.toUpperCase())
			.slice(0, 2)
			.join('');
	}

	const filtered = $derived.by<Artist[]>(() => {
		const q = query.trim().toLowerCase();
		if (!q) return $artistsStore.artists;
		return $artistsStore.artists.filter(
			(a) =>
				a.name.toLowerCase().includes(q) || (a.roles ?? []).some((r) => r.toLowerCase().includes(q))
		);
	});
</script>

<svelte:head>
	<title>Mhaol Cloud — Artists</title>
</svelte:head>

<div class="flex min-h-full flex-col gap-4 p-6">
	<header class="flex flex-wrap items-end justify-between gap-3">
		<div class="flex flex-col gap-1">
			<h1 class="text-2xl font-bold">Artists</h1>
			<p class="text-xs text-base-content/60">
				Every content-addressed <code>artist</code> record persisted in the cloud SurrealDB and pinned
				to IPFS. Firkins reference these by CID.
			</p>
		</div>
		<div class="flex items-center gap-2">
			<input
				type="text"
				class="input-bordered input input-sm"
				placeholder="Filter by name or role"
				bind:value={query}
			/>
			<button
				type="button"
				class="btn btn-outline btn-sm"
				onclick={() => artistsService.refresh()}
				disabled={$artistsStore.loading}
			>
				{$artistsStore.loading ? 'Refreshing…' : 'Refresh'}
			</button>
		</div>
	</header>

	{#if $artistsStore.error}
		<div class="alert alert-error">
			<span>{$artistsStore.error}</span>
		</div>
	{/if}

	{#if $artistsStore.loading && $artistsStore.artists.length === 0}
		<p class="text-sm text-base-content/60">Loading…</p>
	{:else if $artistsStore.artists.length === 0}
		<p class="text-sm text-base-content/60">
			No artists yet. Bookmark a catalog item that has cast/crew, an album with credits, a YouTube
			video, or any other addon item — its artists will be upserted and show up here.
		</p>
	{:else if filtered.length === 0}
		<p class="text-sm text-base-content/60">No matches for “{query}”.</p>
	{:else}
		<ul class="grid grid-cols-1 gap-2 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
			{#each filtered as artist (artist.id)}
				<li>
					<a
						href="{base}/artist/{encodeURIComponent(artist.id)}"
						class="flex items-center gap-3 rounded-box border border-base-content/10 bg-base-200 p-3 hover:border-base-content/30"
					>
						{#if artist.imageUrl}
							<img
								src={artist.imageUrl}
								alt={artist.name}
								class="h-14 w-14 shrink-0 rounded-full object-cover"
								loading="lazy"
							/>
						{:else}
							<span
								class="flex h-14 w-14 shrink-0 items-center justify-center rounded-full bg-base-300 text-sm font-semibold text-base-content/60"
							>
								{initials(artist.name)}
							</span>
						{/if}
						<div class="flex min-w-0 flex-1 flex-col gap-1">
							<span class="truncate text-sm font-medium">{artist.name}</span>
							{#if artist.roles && artist.roles.length > 0}
								<div class="flex flex-wrap gap-1">
									{#each artist.roles.slice(0, 3) as role (role)}
										<span class="badge badge-ghost badge-sm text-[10px]">{role}</span>
									{/each}
									{#if artist.roles.length > 3}
										<span class="badge badge-ghost badge-sm text-[10px]"
											>+{artist.roles.length - 3}</span
										>
									{/if}
								</div>
							{/if}
							<span class="truncate font-mono text-[10px] text-base-content/40">{artist.id}</span>
						</div>
					</a>
				</li>
			{/each}
		</ul>
	{/if}
</div>
