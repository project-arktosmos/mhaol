<script lang="ts">
	import { base } from '$app/paths';
	import { loadMusicbrainzAlbumsByArtist, type CatalogItem } from '$lib/catalog.service';
	import { materializeBrowseFirkin } from '$lib/catalog-firkin';
	import { cachedImageUrl } from '$lib/image-cache';

	interface Props {
		releaseGroupId: string | null;
	}

	let { releaseGroupId }: Props = $props();

	type Status = 'idle' | 'loading' | 'done' | 'error';
	let status = $state<Status>('idle');
	let error = $state<string | null>(null);
	let items = $state<CatalogItem[]>([]);
	let firkinIds = $state<Record<string, string>>({});
	let loadedKey: string | null = null;

	$effect(() => {
		if (!releaseGroupId) {
			items = [];
			firkinIds = {};
			status = 'idle';
			error = null;
			return;
		}
		const key = releaseGroupId;
		if (loadedKey === key) return;
		loadedKey = key;
		void load(key);
	});

	async function load(currentId: string) {
		status = 'loading';
		error = null;
		items = [];
		firkinIds = {};
		try {
			const fetched = await loadMusicbrainzAlbumsByArtist(currentId);
			if (loadedKey !== currentId) return;
			items = fetched;
			status = 'done';
			void materializeAll(currentId, fetched);
		} catch (err) {
			if (loadedKey !== currentId) return;
			error = err instanceof Error ? err.message : 'Unknown error';
			status = 'error';
		}
	}

	async function materializeAll(currentId: string, list: CatalogItem[]) {
		await Promise.all(
			list.map(async (item) => {
				try {
					const created = await materializeBrowseFirkin({
						addon: 'musicbrainz',
						upstreamId: item.id,
						title: item.title,
						year: item.year,
						description: item.description,
						posterUrl: item.posterUrl,
						backdropUrl: item.backdropUrl,
						artistName: item.artistName,
						reviews: item.reviews
					});
					if (loadedKey !== currentId) return;
					firkinIds = { ...firkinIds, [item.id]: created.id };
				} catch (err) {
					console.warn('[albums-by-artist] failed to materialize firkin for', item.id, err);
				}
			})
		);
	}

	function hrefFor(item: CatalogItem): string | undefined {
		const id = firkinIds[item.id];
		return id ? `${base}/catalog/${encodeURIComponent(id)}` : undefined;
	}
</script>

{#if releaseGroupId}
	<div class="card border border-base-content/10 bg-base-200">
		<div class="card-body p-4">
			<h2 class="text-sm font-semibold text-base-content/70 uppercase">More by this artist</h2>

			{#if status === 'loading'}
				<div class="flex items-center gap-2 text-xs text-base-content/60">
					<span class="loading loading-xs loading-spinner"></span>
					<span>Loading albums…</span>
				</div>
			{:else if status === 'error'}
				<div class="alert alert-error">
					<span class="text-xs">{error ?? 'Failed to load albums'}</span>
				</div>
			{:else if status === 'done' && items.length === 0}
				<p class="text-xs text-base-content/60">No other albums found for this artist.</p>
			{:else if items.length > 0}
				<ul class="flex flex-col gap-2">
					{#each items as item (item.id)}
						{@const href = hrefFor(item)}
						<li>
							<a
								{href}
								title={item.title}
								class="group flex items-center gap-3 rounded p-1 transition-colors hover:bg-base-300"
								class:pointer-events-none={!href}
								class:opacity-60={!href}
								aria-disabled={!href}
							>
								<div
									class="aspect-square h-12 w-12 shrink-0 overflow-hidden rounded border border-base-content/10 bg-base-300"
								>
									{#if item.posterUrl}
										<img
											src={cachedImageUrl(item.posterUrl)}
											alt={item.title}
											class="h-full w-full object-cover"
											loading="lazy"
										/>
									{:else}
										<div
											class="flex h-full w-full items-center justify-center text-base-content/20"
										>
											<svg class="h-5 w-5" fill="currentColor" viewBox="0 0 24 24">
												<path
													d="M21 3H3c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h18c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm0 16H3V5h18v14z"
												/>
											</svg>
										</div>
									{/if}
								</div>
								<div class="flex min-w-0 flex-col gap-0.5">
									<span class="line-clamp-2 text-xs leading-snug font-medium">
										{item.title}
									</span>
									{#if item.year !== null && item.year !== undefined}
										<span class="text-xs text-base-content/60">{item.year}</span>
									{/if}
								</div>
							</a>
						</li>
					{/each}
				</ul>
			{/if}
		</div>
	</div>
{/if}
