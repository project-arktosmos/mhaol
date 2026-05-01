<script lang="ts">
	import { onMount } from 'svelte';
	import classNames from 'classnames';
	import { base } from '$app/paths';
	import { goto } from '$app/navigation';
	import FirkinCard from '$components/firkins/FirkinCard.svelte';
	import FirkinMetadataLookupModal, {
		type CatalogLookupItem
	} from '$components/firkins/FirkinMetadataLookupModal.svelte';
	import type { CloudFirkin } from '$types/firkin.type';
	import {
		listSources,
		loadGenres,
		loadPopular,
		loadSearch,
		type CatalogItem,
		type CatalogGenre,
		type CatalogSource
	} from '$lib/catalog.service';
	import { firkinsService, metadataSearchAddon, type Firkin } from '$lib/firkins.service';

	const firkinsStore = firkinsService.state;

	let sources = $state<CatalogSource[]>([]);
	let sourcesError = $state<string | null>(null);

	let addon = $state<string>('');
	let filter = $state<string>('');
	let page = $state<number>(1);

	let genres = $state<CatalogGenre[]>([]);
	let genresLoading = $state(false);
	let genresError = $state<string | null>(null);

	let items = $state<CatalogItem[]>([]);
	let totalPages = $state<number>(1);
	let itemsLoading = $state(false);
	let itemsError = $state<string | null>(null);

	let query = $state<string>('');
	let searchItems = $state<CatalogItem[]>([]);
	let searchPage = $state<number>(1);
	let searchTotalPages = $state<number>(1);
	let searchLoading = $state(false);
	let searchError = $state<string | null>(null);
	let searchToken = 0;
	let searchDebounce: ReturnType<typeof setTimeout> | null = null;
	const trimmedQuery = $derived(query.trim());
	const hasSearch = $derived(trimmedQuery.length > 0);

	const currentSource = $derived(sources.find((s) => s.id === addon));
	const filterLabel = $derived(currentSource?.filterLabel ?? 'Filter');
	const hasFilter = $derived(currentSource?.hasFilter ?? false);
	const showFilterRow = $derived(hasFilter);

	// Each catalog (remote) addon has a matching local-* addon used by
	// library scans for the same content kind. The catalog Library section
	// should surface both: virtual / bookmarked items live under the remote
	// addon, locally-scanned files live under the local-* counterpart.
	const LOCAL_ADDON_FOR: Record<string, string> = {
		'tmdb-movie': 'local-movie',
		'tmdb-tv': 'local-tv',
		musicbrainz: 'local-album'
	};

	const libraryFirkins = $derived<Firkin[]>(
		addon
			? $firkinsStore.firkins
					.filter((d) => d.addon === addon || d.addon === LOCAL_ADDON_FOR[addon])
					.slice()
					.sort((a, b) => b.created_at.localeCompare(a.created_at))
					.slice(0, 6)
			: []
	);

	function virtualFirkin(item: CatalogItem): CloudFirkin {
		const images = [item.posterUrl, item.backdropUrl]
			.filter((url): url is string => Boolean(url))
			.map((url) => ({ url, mimeType: 'image/jpeg', fileSize: 0, width: 0, height: 0 }));
		return {
			id: `virtual:${addon}:${item.id}`,
			title: item.title,
			artists: [],
			description: item.description ?? '',
			images,
			files: [],
			year: item.year,
			addon,
			creator: '',
			created_at: '',
			updated_at: '',
			version: 0,
			version_hashes: []
		};
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

	async function refreshGenres() {
		if (!addon || !hasFilter) {
			genres = [];
			filter = '';
			return;
		}
		genresLoading = true;
		genresError = null;
		try {
			genres = await loadGenres(addon);
			if (!genres.some((g) => g.id === filter)) {
				filter = genres[0]?.id ?? '';
			}
		} catch (err) {
			genres = [];
			genresError = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			genresLoading = false;
		}
	}

	async function runSearch(nextPage = 1) {
		if (!addon || !trimmedQuery) {
			searchItems = [];
			searchTotalPages = 1;
			searchPage = 1;
			searchError = null;
			return;
		}
		const token = ++searchToken;
		searchLoading = true;
		searchError = null;
		try {
			const result = await loadSearch(addon, trimmedQuery, {
				filter: filter || undefined,
				page: nextPage
			});
			if (token !== searchToken) return;
			searchItems = result.items;
			searchTotalPages = result.totalPages;
			searchPage = result.page;
		} catch (err) {
			if (token !== searchToken) return;
			searchItems = [];
			searchTotalPages = 1;
			searchError = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			if (token === searchToken) searchLoading = false;
		}
	}

	function scheduleSearch() {
		if (searchDebounce) clearTimeout(searchDebounce);
		if (!trimmedQuery) {
			searchToken++;
			searchItems = [];
			searchTotalPages = 1;
			searchPage = 1;
			searchError = null;
			searchLoading = false;
			return;
		}
		searchDebounce = setTimeout(() => {
			void runSearch(1);
		}, 300);
	}

	async function goToSearchPage(next: number) {
		if (next < 1 || next > searchTotalPages || next === searchPage) return;
		await runSearch(next);
	}

	async function refreshItems() {
		if (!addon) {
			items = [];
			return;
		}
		itemsLoading = true;
		itemsError = null;
		try {
			const result = await loadPopular(addon, {
				filter: filter || undefined,
				page
			});
			items = result.items;
			totalPages = result.totalPages;
			page = result.page;
		} catch (err) {
			items = [];
			totalPages = 1;
			itemsError = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			itemsLoading = false;
		}
	}

	async function selectAddon(source: CatalogSource) {
		if (addon === source.id) return;
		addon = source.id;
		page = 1;
		filter = '';
		query = '';
		searchToken++;
		searchItems = [];
		searchTotalPages = 1;
		searchPage = 1;
		searchError = null;
		searchLoading = false;
		await refreshGenres();
		await refreshItems();
	}

	async function onFilterChange() {
		page = 1;
		await refreshItems();
		if (trimmedQuery) await runSearch(1);
	}

	// Library firkins are flagged as "needs metadata" when they're missing
	// the two fields a catalog match would normally provide: a description
	// and at least one image. Year alone isn't enough — local-* scanners
	// often parse `(YYYY)` out of the filename so a freshly-scanned firkin
	// can have a year but no other metadata.
	function firkinNeedsMetadata(firkin: Firkin): boolean {
		return firkin.description.trim() === '' || firkin.images.length === 0;
	}

	let metadataTarget = $state<{ firkin: Firkin; addon: string } | null>(null);

	function openMetadataLookup(firkin: Firkin) {
		const addonId = metadataSearchAddon(firkin.addon);
		if (!addonId) return;
		metadataTarget = { firkin, addon: addonId };
	}

	async function applyMetadata(item: CatalogLookupItem) {
		if (!metadataTarget) return;
		const oldId = metadataTarget.firkin.id;
		const updated = await firkinsService.enrich(oldId, {
			title: item.title,
			year: item.year,
			description: item.description ?? '',
			posterUrl: item.posterUrl,
			backdropUrl: item.backdropUrl
		});
		metadataTarget = null;
		// Refresh so the rolled-forward firkin (new CID) replaces the old
		// one in the Library section's list.
		await firkinsService.refresh();
		if (updated.id !== oldId) {
			void goto(`${base}/catalog/${encodeURIComponent(updated.id)}`);
		}
	}

	async function goToPage(next: number) {
		if (next < 1 || next > totalPages || next === page) return;
		page = next;
		await refreshItems();
	}

	onMount(() => {
		const stopFirkins = firkinsService.start();
		void (async () => {
			try {
				const fetched = await listSources();
				sources = fetched.filter((s) => s.id !== 'youtube-video' && s.id !== 'youtube-channel');
				if (sources.length > 0) {
					addon = sources[0].id;
					await refreshGenres();
					await refreshItems();
				}
			} catch (err) {
				sourcesError = err instanceof Error ? err.message : 'Unknown error';
			}
		})();
		return () => {
			stopFirkins();
		};
	});
</script>

<svelte:head>
	<title>Mhaol Cloud — Catalog</title>
</svelte:head>

<div class="flex min-h-full flex-col gap-6 p-6">
	{#if sourcesError}
		<div class="alert alert-error">
			<span>Could not load catalog sources: {sourcesError}</span>
		</div>
	{/if}

	<section class="card border border-base-content/10 bg-base-200 p-4">
		<div class="overflow-x-auto rounded-box border border-base-content/10">
			<table class="table table-sm">
				<tbody>
					<tr>
						<th class="w-32 align-middle">Addon</th>
						<td>
							<div class="flex flex-wrap gap-2">
								{#each sources as source (source.id)}
									{@const active = addon === source.id}
									<button
										type="button"
										class={classNames('btn btn-sm', {
											'btn-primary': active,
											'btn-outline': !active
										})}
										onclick={() => selectAddon(source)}
										title={source.kind}
									>
										{source.label}
									</button>
								{/each}
							</div>
						</td>
					</tr>
					<tr>
						<th class="w-32 align-middle">Search</th>
						<td>
							<input
								type="search"
								class="input-bordered input input-sm w-full"
								placeholder={addon
									? `Search ${currentSource?.label ?? addon}…`
									: 'Pick an addon to search'}
								disabled={!addon}
								bind:value={query}
								oninput={scheduleSearch}
							/>
						</td>
					</tr>
					{#if showFilterRow}
						<tr>
							<th class="w-32 align-middle">{filterLabel}</th>
							<td>
								{#if genresLoading}
									<span class="text-xs text-base-content/60"
										>Loading {filterLabel.toLowerCase()}…</span
									>
								{:else if genresError}
									<span class="text-xs text-error">{genresError}</span>
								{:else if genres.length === 0}
									<span class="text-xs text-base-content/60">No options available</span>
								{:else}
									<select
										class="select-bordered select w-full select-sm"
										bind:value={filter}
										onchange={onFilterChange}
									>
										{#each genres as option (option.id)}
											<option value={option.id}>{option.name}</option>
										{/each}
									</select>
								{/if}
							</td>
						</tr>
					{/if}
				</tbody>
			</table>
		</div>
	</section>

	<section class="flex flex-col gap-3">
		<h2 class="text-lg font-semibold">Library</h2>
		{#if libraryFirkins.length === 0}
			<p class="text-sm text-base-content/60">No library items yet.</p>
		{:else}
			<div class="grid grid-cols-6 gap-4">
				{#each libraryFirkins as doc (doc.id)}
					{@const canEnrich = firkinNeedsMetadata(doc) && metadataSearchAddon(doc.addon) !== null}
					<div class="relative">
						<a
							href={`${base}/catalog/${encodeURIComponent(doc.id)}`}
							class="block no-underline"
							onclick={(e) => {
								if ((e.target as HTMLElement).closest('button, summary')) {
									e.preventDefault();
								}
							}}
						>
							<FirkinCard firkin={doc as CloudFirkin} />
						</a>
						{#if canEnrich}
							<button
								type="button"
								class="btn absolute top-2 right-2 btn-xs btn-primary"
								onclick={() => openMetadataLookup(doc)}
								title="Search the relevant addon and bake matching metadata into this firkin"
							>
								Find metadata
							</button>
						{/if}
					</div>
				{/each}
			</div>
		{/if}
	</section>

	{#if hasSearch}
		<section class="flex flex-col gap-3">
			<div class="flex items-center justify-between gap-4">
				<h2 class="text-lg font-semibold">Search results</h2>
				<div class="flex items-center gap-2">
					<button
						class="btn btn-outline btn-xs"
						onclick={() => goToSearchPage(searchPage - 1)}
						disabled={searchLoading || searchPage <= 1}
					>
						Prev
					</button>
					<span class="text-xs text-base-content/60">Page {searchPage} / {searchTotalPages}</span>
					<button
						class="btn btn-outline btn-xs"
						onclick={() => goToSearchPage(searchPage + 1)}
						disabled={searchLoading || searchPage >= searchTotalPages}
					>
						Next
					</button>
				</div>
			</div>

			{#if searchError}
				<div class="alert alert-error">
					<span>{searchError}</span>
				</div>
			{/if}

			{#if searchLoading && searchItems.length === 0}
				<p class="text-sm text-base-content/60">Searching…</p>
			{:else if searchItems.length === 0}
				<p class="text-sm text-base-content/60">No matches.</p>
			{:else}
				<div
					class={classNames(
						'grid grid-cols-1 gap-4 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5',
						{ 'opacity-60': searchLoading }
					)}
				>
					{#each searchItems as item (item.id)}
						<a
							href={virtualHref(item)}
							class="block no-underline"
							onclick={(e) => {
								if ((e.target as HTMLElement).closest('button, summary')) {
									e.preventDefault();
								}
							}}
						>
							<FirkinCard firkin={virtualFirkin(item)} />
						</a>
					{/each}
				</div>
			{/if}
		</section>
	{/if}

	<section class="flex flex-col gap-3">
		<div class="flex items-center justify-between gap-4">
			<h2 class="text-lg font-semibold">Popular</h2>
			<div class="flex items-center gap-2">
				<button
					class="btn btn-outline btn-xs"
					onclick={() => goToPage(page - 1)}
					disabled={itemsLoading || page <= 1}
				>
					Prev
				</button>
				<span class="text-xs text-base-content/60">Page {page} / {totalPages}</span>
				<button
					class="btn btn-outline btn-xs"
					onclick={() => goToPage(page + 1)}
					disabled={itemsLoading || page >= totalPages}
				>
					Next
				</button>
				<button class="btn btn-outline btn-xs" onclick={refreshItems} disabled={itemsLoading}>
					Refresh
				</button>
			</div>
		</div>

		{#if itemsError}
			<div class="alert alert-error">
				<span>{itemsError}</span>
			</div>
		{/if}

		{#if itemsLoading && items.length === 0}
			<p class="text-sm text-base-content/60">Loading…</p>
		{:else if items.length === 0}
			<p class="text-sm text-base-content/60">No items.</p>
		{:else}
			<div
				class={classNames(
					'grid grid-cols-1 gap-4 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5',
					{ 'opacity-60': itemsLoading }
				)}
			>
				{#each items as item (item.id)}
					<a
						href={virtualHref(item)}
						class="block no-underline"
						onclick={(e) => {
							if ((e.target as HTMLElement).closest('button, summary')) {
								e.preventDefault();
							}
						}}
					>
						<FirkinCard firkin={virtualFirkin(item)} />
					</a>
				{/each}
			</div>
		{/if}
	</section>
</div>

{#if metadataTarget}
	<FirkinMetadataLookupModal
		open={metadataTarget !== null}
		addon={metadataTarget.addon}
		initialQuery={metadataTarget.firkin.title}
		firkinTitle={metadataTarget.firkin.title}
		onpick={applyMetadata}
		onclose={() => (metadataTarget = null)}
	/>
{/if}
