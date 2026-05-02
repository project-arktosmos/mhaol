<script lang="ts">
	import { base } from '$app/paths';
	import { loadRelated, type CatalogItem } from '$lib/catalog.service';
	import { materializeBrowseFirkin } from '$lib/catalog-firkin';
	import type { FirkinAddon } from '$lib/firkins.service';
	import { cachedImageUrl } from '$lib/image-cache';

	interface Props {
		addon: string;
		upstreamId: string | null;
		/**
		 * Fires once per `(addon, upstreamId)` change when the related list
		 * has loaded. Used by `/catalog/[id]` to feed the items into the
		 * per-user recommendation counter; only invoked when the source
		 * firkin is bookmarked (the catalog detail page checks
		 * `firkin.bookmarked` before forwarding the items).
		 */
		onItemsLoaded?: (items: CatalogItem[]) => void;
	}

	let { addon, upstreamId, onItemsLoaded }: Props = $props();

	const isMusicBrainz = $derived(addon === 'musicbrainz');

	type Status = 'idle' | 'loading' | 'done' | 'error';
	let status = $state<Status>('idle');
	let error = $state<string | null>(null);
	let items = $state<CatalogItem[]>([]);
	let firkinIds = $state<Record<string, string>>({});
	let loadedKey: string | null = null;

	$effect(() => {
		if (!addon || !upstreamId) {
			items = [];
			firkinIds = {};
			status = 'idle';
			error = null;
			return;
		}
		const key = `${addon}:${upstreamId}`;
		if (loadedKey === key) return;
		loadedKey = key;
		void load(addon, upstreamId);
	});

	async function load(currentAddon: string, currentId: string) {
		status = 'loading';
		error = null;
		items = [];
		firkinIds = {};
		try {
			const fetched = await loadRelated(currentAddon, currentId);
			if (loadedKey !== `${currentAddon}:${currentId}`) return;
			items = fetched;
			status = 'done';
			onItemsLoaded?.(fetched);
			void materializeAll(currentAddon, currentId, fetched);
		} catch (err) {
			if (loadedKey !== `${currentAddon}:${currentId}`) return;
			error = err instanceof Error ? err.message : 'Unknown error';
			status = 'error';
		}
	}

	async function materializeAll(currentAddon: string, currentId: string, list: CatalogItem[]) {
		await Promise.all(
			list.map(async (item) => {
				try {
					const created = await materializeBrowseFirkin({
						addon: currentAddon as FirkinAddon,
						upstreamId: item.id,
						title: item.title,
						year: item.year,
						description: item.description,
						posterUrl: item.posterUrl,
						backdropUrl: item.backdropUrl,
						artistName: item.artistName,
						reviews: item.reviews
					});
					if (loadedKey !== `${currentAddon}:${currentId}`) return;
					firkinIds = { ...firkinIds, [item.id]: created.id };
				} catch (err) {
					console.warn('[related] failed to materialize firkin for', item.id, err);
				}
			})
		);
	}

	function hrefFor(item: CatalogItem): string | undefined {
		const id = firkinIds[item.id];
		return id ? `${base}/catalog/${encodeURIComponent(id)}` : undefined;
	}
</script>

{#if upstreamId}
	<div class="card border border-base-content/10 bg-base-200">
		<div class="card-body p-4">
			<h2 class="text-sm font-semibold text-base-content/70 uppercase">Related</h2>

			{#if status === 'loading'}
				<div class="flex items-center gap-2 text-xs text-base-content/60">
					<span class="loading loading-xs loading-spinner"></span>
					<span>Loading related items…</span>
				</div>
			{:else if status === 'error'}
				<div class="alert alert-error">
					<span class="text-xs">{error ?? 'Failed to load related items'}</span>
				</div>
			{:else if status === 'done' && items.length === 0}
				<p class="text-xs text-base-content/60">No related items found upstream.</p>
			{:else if items.length > 0 && isMusicBrainz}
				<ul class="flex flex-col gap-2">
					{#each items as item (item.id)}
						{@const href = hrefFor(item)}
						<li>
							<a
								{href}
								title={item.title}
								class="group flex items-start gap-3 rounded transition-all"
								class:pointer-events-none={!href}
								class:opacity-60={!href}
								aria-disabled={!href}
							>
								<div
									class="aspect-square w-16 shrink-0 overflow-hidden rounded border border-base-content/10 bg-base-300 transition-all group-hover:border-base-content/30 group-hover:shadow-md"
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
											<svg class="h-6 w-6" fill="currentColor" viewBox="0 0 24 24">
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
									{#if item.artistName}
										<span class="line-clamp-1 text-xs text-base-content/60">
											{item.artistName}
										</span>
									{/if}
								</div>
							</a>
						</li>
					{/each}
				</ul>
			{:else if items.length > 0}
				<ul class="grid grid-cols-3 gap-3 sm:grid-cols-4 lg:grid-cols-2">
					{#each items as item (item.id)}
						{@const href = hrefFor(item)}
						<li>
							<a
								{href}
								title={item.title}
								class="group flex flex-col gap-1.5 rounded transition-all"
								class:pointer-events-none={!href}
								class:opacity-60={!href}
								aria-disabled={!href}
							>
								<div
									class="aspect-[2/3] overflow-hidden rounded border border-base-content/10 bg-base-300 transition-all group-hover:border-base-content/30 group-hover:shadow-md"
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
											<svg class="h-8 w-8" fill="currentColor" viewBox="0 0 24 24">
												<path
													d="M21 3H3c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h18c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm0 16H3V5h18v14z"
												/>
											</svg>
										</div>
									{/if}
								</div>
								<div class="flex flex-col gap-0.5">
									<span class="line-clamp-2 text-xs leading-snug font-medium">
										{item.title}
									</span>
									{#if item.artistName}
										<span class="line-clamp-1 text-xs text-base-content/60">
											{item.artistName}
										</span>
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
