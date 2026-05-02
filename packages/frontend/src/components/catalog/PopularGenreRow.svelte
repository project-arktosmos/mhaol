<script lang="ts">
	import { base } from '$app/paths';
	import FirkinLibraryGrid from '$components/catalog/FirkinLibraryGrid.svelte';
	import { loadPopular, type CatalogItem } from '$lib/catalog.service';
	import type { CloudFirkin } from '$types/firkin.type';

	interface Props {
		addon: string;
		genreId?: string;
		title: string;
		hrefBuilder: (firkin: CloudFirkin) => string;
	}

	let { addon, genreId, title, hrefBuilder }: Props = $props();

	let items = $state<CatalogItem[]>([]);
	let loading = $state(false);
	let error = $state<string | null>(null);

	const moreHref = $derived(
		genreId
			? `${base}/catalog/gallery?addon=${encodeURIComponent(addon)}&mode=popular&filter=${encodeURIComponent(genreId)}`
			: `${base}/catalog/gallery?addon=${encodeURIComponent(addon)}&mode=popular`
	);

	function virtualFirkin(item: CatalogItem): CloudFirkin {
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
			id: `virtual:${addon}:${item.id}`,
			cid: '',
			title: item.title,
			artists,
			description: item.description ?? '',
			images,
			files: [],
			year: item.year,
			addon,
			creator: '',
			created_at: '',
			updated_at: '',
			version: 0,
			version_hashes: [],
			reviews: item.reviews ?? []
		};
	}

	const firkins = $derived<CloudFirkin[]>(items.map((it) => virtualFirkin(it)));

	$effect(() => {
		if (!addon) return;
		void addon;
		void genreId;
		loading = true;
		error = null;
		void (async () => {
			try {
				const result = await loadPopular(addon, {
					filter: genreId || undefined,
					page: 1
				});
				items = result.items;
			} catch (err) {
				items = [];
				error = err instanceof Error ? err.message : 'Unknown error';
			} finally {
				loading = false;
			}
		})();
	});
</script>

<section class="flex flex-col gap-3">
	<div class="flex flex-wrap items-center justify-between gap-4">
		<h2 class="text-lg font-semibold">{title}</h2>
	</div>
	{#if error}
		<div class="alert alert-error"><span>{error}</span></div>
	{/if}
	{#if loading && items.length === 0}
		<p class="text-sm text-base-content/60">Loading…</p>
	{:else}
		<div class={loading ? 'opacity-60' : ''}>
			<FirkinLibraryGrid
				firkins={firkins}
				collapsed={true}
				collapsedCount={6}
				moreHref={moreHref}
				hrefBuilder={hrefBuilder}
				emptyMessage="No items."
			/>
		</div>
	{/if}
</section>
