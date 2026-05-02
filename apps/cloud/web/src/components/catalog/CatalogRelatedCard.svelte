<script lang="ts">
	import { base } from '$app/paths';
	import { loadRelated, type CatalogItem } from '$lib/catalog.service';

	interface Props {
		addon: string;
		upstreamId: string | null;
		/**
		 * Fires once per `(addon, upstreamId)` change when the related list
		 * has loaded. Used by `/catalog/[ipfsHash]` to feed the items into
		 * the per-user recommendation counter; not used by `/catalog/virtual`
		 * (virtual pages must not update counts).
		 */
		onItemsLoaded?: (items: CatalogItem[]) => void;
	}

	let { addon, upstreamId, onItemsLoaded }: Props = $props();

	type Status = 'idle' | 'loading' | 'done' | 'error';
	let status = $state<Status>('idle');
	let error = $state<string | null>(null);
	let items = $state<CatalogItem[]>([]);
	let loadedKey: string | null = null;

	$effect(() => {
		if (!addon || !upstreamId) {
			items = [];
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
		try {
			const fetched = await loadRelated(currentAddon, currentId);
			if (loadedKey !== `${currentAddon}:${currentId}`) return;
			items = fetched;
			status = 'done';
			onItemsLoaded?.(fetched);
		} catch (err) {
			if (loadedKey !== `${currentAddon}:${currentId}`) return;
			error = err instanceof Error ? err.message : 'Unknown error';
			status = 'error';
		}
	}

	function virtualHref(item: CatalogItem): string {
		const params = new URLSearchParams();
		params.set('addon', addon);
		params.set('id', item.id);
		params.set('title', item.title);
		if (item.year !== null && item.year !== undefined) params.set('year', String(item.year));
		if (item.description) params.set('description', item.description);
		if (item.posterUrl) params.set('posterUrl', item.posterUrl);
		if (item.backdropUrl) params.set('backdropUrl', item.backdropUrl);
		return `${base}/catalog/virtual?${params.toString()}`;
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
			{:else if items.length > 0}
				<ul class="grid grid-cols-3 gap-3 sm:grid-cols-4 lg:grid-cols-2">
					{#each items as item (item.id)}
						<li>
							<a
								href={virtualHref(item)}
								title={item.title}
								class="group flex flex-col gap-1.5 rounded transition-all"
							>
								<div
									class="aspect-[2/3] overflow-hidden rounded border border-base-content/10 bg-base-300 transition-all group-hover:border-base-content/30 group-hover:shadow-md"
								>
									{#if item.posterUrl}
										<img
											src={item.posterUrl}
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
									<span class="line-clamp-2 text-xs font-medium leading-snug">
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
