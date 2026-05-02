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

	// FirkinLibraryGrid renders a fixed 7-col grid: 6 firkin cards + 1 "More"
	// cell. The row only looks complete when there are at least 7 items (the
	// 7th overflows and produces the "More" link); fewer items leave empty
	// cells to the right. Anything below this threshold is hidden so the page
	// never shows a half-filled or empty genre row.
	const MIN_ITEMS_TO_SHOW = 7;
	// TMDB popular pages return 20 items each, but the backend filters out
	// items the user already has firkins for — sparse genres can drop most of
	// a page. Pull a few pages so we usually have enough to fill the row
	// before giving up and hiding it.
	const MAX_PAGES_TO_FETCH = 5;

	let items = $state<CatalogItem[]>([]);
	let loaded = $state(false);
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
	const showRow = $derived(items.length >= MIN_ITEMS_TO_SHOW);

	$effect(() => {
		if (!addon) return;
		void addon;
		void genreId;
		loaded = false;
		error = null;
		items = [];
		void (async () => {
			try {
				const collected: CatalogItem[] = [];
				const seen = new Set<string>();
				let page = 1;
				let totalPages = 1;
				while (collected.length < MIN_ITEMS_TO_SHOW && page <= MAX_PAGES_TO_FETCH) {
					const result = await loadPopular(addon, {
						filter: genreId || undefined,
						page
					});
					totalPages = result.totalPages;
					for (const it of result.items) {
						if (seen.has(it.id)) continue;
						seen.add(it.id);
						collected.push(it);
					}
					if (page >= totalPages) break;
					page += 1;
				}
				items = collected;
			} catch (err) {
				items = [];
				error = err instanceof Error ? err.message : 'Unknown error';
			} finally {
				loaded = true;
			}
		})();
	});
</script>

{#if error}
	<section class="flex flex-col gap-3">
		<div class="flex flex-wrap items-center justify-between gap-4">
			<h2 class="text-lg font-semibold">{title}</h2>
		</div>
		<div class="alert alert-error"><span>{error}</span></div>
	</section>
{:else if loaded && showRow}
	<section class="flex flex-col gap-3">
		<div class="flex flex-wrap items-center justify-between gap-4">
			<h2 class="text-lg font-semibold">{title}</h2>
		</div>
		<FirkinLibraryGrid
			{firkins}
			collapsed={true}
			collapsedCount={6}
			{moreHref}
			{hrefBuilder}
			emptyMessage="No items."
		/>
	</section>
{/if}
