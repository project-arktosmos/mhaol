<script lang="ts">
	import { base } from '$app/paths';
	import { goto } from '$app/navigation';
	import { loadRelated, type CatalogItem } from '$lib/catalog.service';
	import { materializeBrowseFirkin } from '$lib/catalog-firkin';
	import type { FirkinAddon } from '$lib/firkins.service';
	import FirkinCard from '$components/firkins/FirkinCard.svelte';
	import type { CloudFirkin } from '$types/firkin.type';

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

	function hrefFor(item: CatalogItem): string {
		const id = firkinIds[item.id];
		return id ? `${base}/catalog/${encodeURIComponent(id)}` : `${base}/catalog/visit`;
	}

	function toFirkin(item: CatalogItem): CloudFirkin {
		const images = [item.posterUrl, item.backdropUrl]
			.filter((url): url is string => Boolean(url))
			.map((url) => ({ url, mimeType: 'image/jpeg', fileSize: 0, width: 0, height: 0 }));
		const artists = item.artistName
			? item.artistName
					.split(/\s*,\s*/)
					.filter((n) => n.length > 0)
					.map((name) => ({ name, role: 'artist' }))
			: [];
		return {
			id: firkinIds[item.id] ?? `virtual:${addon}:${item.id}`,
			cid: '',
			title: item.title,
			artists,
			description: item.description ?? '',
			images,
			files: [],
			year: item.year,
			addon: addon as FirkinAddon,
			creator: '',
			created_at: '',
			updated_at: '',
			version: 0,
			version_hashes: [],
			reviews: item.reviews ?? []
		};
	}

	async function handleClick(event: MouseEvent, item: CatalogItem) {
		if (event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) {
			return;
		}
		event.preventDefault();
		let id = firkinIds[item.id];
		if (!id) {
			try {
				const created = await materializeBrowseFirkin({
					addon: addon as FirkinAddon,
					upstreamId: item.id,
					title: item.title,
					year: item.year,
					description: item.description,
					posterUrl: item.posterUrl,
					backdropUrl: item.backdropUrl,
					artistName: item.artistName,
					reviews: item.reviews
				});
				id = created.id;
				firkinIds = { ...firkinIds, [item.id]: id };
			} catch (err) {
				console.warn('[related] click materialize failed for', item.id, err);
				return;
			}
		}
		await goto(`${base}/catalog/${encodeURIComponent(id)}`);
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
				<div class="grid grid-cols-2 gap-3">
					{#each items as item (item.id)}
						<a
							href={hrefFor(item)}
							onclick={(e) => handleClick(e, item)}
							class="block no-underline"
						>
							<FirkinCard firkin={toFirkin(item)} />
						</a>
					{/each}
				</div>
			{/if}
		</div>
	</div>
{/if}
