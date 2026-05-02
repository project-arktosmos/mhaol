<script lang="ts">
	import { onMount } from 'svelte';
	import classNames from 'classnames';
	import { base } from '$app/paths';
	import { goto } from '$app/navigation';
	import { page as pageStore } from '$app/state';
	import FirkinCard from '$components/firkins/FirkinCard.svelte';
	import FirkinLibraryGrid from '$components/catalog/FirkinLibraryGrid.svelte';
	import LazyRow from '$components/catalog/LazyRow.svelte';
	import PopularGenreRow from '$components/catalog/PopularGenreRow.svelte';
	import FirkinMetadataLookupModal, {
		type CatalogLookupItem
	} from '$components/firkins/FirkinMetadataLookupModal.svelte';
	import type { CloudFirkin } from '$types/firkin.type';
	import {
		listSources,
		loadGenres,
		loadSearch,
		type CatalogItem,
		type CatalogGenre,
		type CatalogSource
	} from '$lib/catalog.service';
	import {
		firkinsService,
		metadataSearchAddon,
		type Firkin,
		type FirkinAddon
	} from '$lib/firkins.service';
	import { listRecommendations, type Recommendation } from '$lib/recommendations.service';
	import { userIdentityService } from '$lib/user-identity.service';
	import { artistsModalService } from '$services/artists-modal.service';
	import { consumptionModalService } from '$services/consumption-modal.service';
	import { diskModalService } from '$services/disk-modal.service';
	import { healthModalService } from '$services/health-modal.service';
	import { ipfsModalService } from '$services/ipfs-modal.service';
	import { librariesModalService } from '$services/libraries-modal.service';
	import { torrentModalService } from '$services/torrent-modal.service';

	const firkinsStore = firkinsService.state;
	const firkinsIncludeAll = firkinsService.includeAll;

	let sources = $state<CatalogSource[]>([]);
	let sourcesError = $state<string | null>(null);

	// Selected addon flows from the URL `?addon=<id>` query param. When sources
	// have loaded but the URL has no addon (or names an unknown one), fall back
	// to the first available source so the page is never in a no-selection state.
	const addon = $derived.by(() => {
		const fromUrl = pageStore.url.searchParams.get('addon') ?? '';
		if (sources.length === 0) return fromUrl;
		if (fromUrl && sources.some((s) => s.id === fromUrl)) return fromUrl;
		return sources[0]?.id ?? '';
	});
	// MusicBrainz-only: which release-group field the user wants to search on.
	// Default to artist because the typical free-text query is an artist name
	// ("keane") and the user wants every release-group by that artist back.
	let searchField = $state<'artist' | 'release'>('artist');
	const showSearchFieldSelect = $derived(addon === 'musicbrainz');

	let genres = $state<CatalogGenre[]>([]);

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
	const hasFilter = $derived(currentSource?.hasFilter ?? false);
	const hasPopular = $derived(currentSource?.hasPopular ?? true);

	// Each catalog (remote) addon has a matching local-* addon used by
	// library scans for the same content kind. The catalog Library section
	// should surface both: virtual / bookmarked items live under the remote
	// addon, locally-scanned files live under the local-* counterpart.
	const LOCAL_ADDON_FOR: Record<string, string> = {
		'tmdb-movie': 'local-movie',
		'tmdb-tv': 'local-tv',
		musicbrainz: 'local-album'
	};

	const libraryAllFirkins = $derived<Firkin[]>(
		addon
			? $firkinsStore.firkins
					.filter((d) => d.addon === addon || d.addon === LOCAL_ADDON_FOR[addon])
					.slice()
					.sort((a, b) => b.created_at.localeCompare(a.created_at))
			: []
	);
	const galleryHref = $derived(
		addon ? `${base}/catalog/gallery?addon=${encodeURIComponent(addon)}` : ''
	);
	const recommendationsHref = $derived(
		addon ? `${base}/catalog/gallery?addon=${encodeURIComponent(addon)}&mode=for-you` : ''
	);

	const userIdentityState = userIdentityService.state;
	let recommendations = $state<Recommendation[]>([]);
	let lastRecommendationsAddress: string | null = null;

	$effect(() => {
		const address = $userIdentityState.identity?.address;
		if (!address) {
			recommendations = [];
			lastRecommendationsAddress = null;
			return;
		}
		if (lastRecommendationsAddress === address) return;
		lastRecommendationsAddress = address;
		void (async () => {
			try {
				recommendations = await listRecommendations(address, { excludeActioned: true });
			} catch {
				recommendations = [];
			}
		})();
	});

	const addonRecommendationFirkins = $derived<CloudFirkin[]>(
		addon
			? recommendations.filter((r) => r.addon === addon).map((r) => recommendationToFirkin(r))
			: []
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

	function recommendationToFirkin(row: Recommendation): CloudFirkin {
		const images = [row.posterUrl, row.backdropUrl]
			.filter((url): url is string => Boolean(url))
			.map((url) => ({ url, mimeType: 'image/jpeg', fileSize: 0, width: 0, height: 0 }));
		return {
			id: `virtual:${row.addon}:${row.upstreamId}`,
			cid: row.firkinId,
			title: row.title,
			artists: [],
			description: row.description ?? '',
			images,
			files: [],
			year: row.year,
			addon: row.addon as FirkinAddon,
			creator: '',
			created_at: row.created_at,
			updated_at: row.updated_at,
			version: 0,
			version_hashes: [],
			reviews: row.reviews ?? []
		};
	}

	function visitHref(item: CatalogItem): string {
		const params = new URLSearchParams();
		params.set('addon', addon);
		params.set('id', item.id);
		params.set('title', item.title);
		if (item.year !== null && item.year !== undefined) params.set('year', String(item.year));
		if (item.description) params.set('description', item.description);
		if (item.posterUrl) params.set('posterUrl', item.posterUrl);
		if (item.backdropUrl) params.set('backdropUrl', item.backdropUrl);
		if (item.artistName) params.set('artistName', item.artistName);
		// Forward the upstream review snapshot so the detail page can
		// render it before the metadata-backfill effect refetches.
		if (Array.isArray(item.reviews) && item.reviews.length > 0) {
			params.set('reviews', JSON.stringify(item.reviews));
		}
		return `${base}/catalog/visit?${params.toString()}`;
	}

	function visitHrefForFirkin(firkin: CloudFirkin): string {
		const prefix = `virtual:${firkin.addon}:`;
		const upstreamId = firkin.id.startsWith(prefix) ? firkin.id.slice(prefix.length) : firkin.id;
		const [poster, backdrop] = firkin.images;
		const params = new URLSearchParams();
		params.set('addon', firkin.addon);
		params.set('id', upstreamId);
		params.set('title', firkin.title);
		if (firkin.year !== null && firkin.year !== undefined) {
			params.set('year', String(firkin.year));
		}
		if (firkin.description) params.set('description', firkin.description);
		if (poster?.url) params.set('posterUrl', poster.url);
		if (backdrop?.url) params.set('backdropUrl', backdrop.url);
		const artistNames = (firkin.artists ?? [])
			.map((a) => a.name)
			.filter((n) => n && n.length > 0)
			.join(', ');
		if (artistNames) params.set('artistName', artistNames);
		if (Array.isArray(firkin.reviews) && firkin.reviews.length > 0) {
			params.set('reviews', JSON.stringify(firkin.reviews));
		}
		return `${base}/catalog/visit?${params.toString()}`;
	}

	async function refreshGenres() {
		if (!addon || !hasFilter) {
			genres = [];
			return;
		}
		try {
			genres = await loadGenres(addon);
		} catch {
			genres = [];
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
				page: nextPage,
				field: addon === 'musicbrainz' ? searchField : undefined
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

	async function selectAddon(source: CatalogSource) {
		if (addon === source.id) return;
		const url = new URL(pageStore.url);
		url.searchParams.set('addon', source.id);
		await goto(`${url.pathname}${url.search}`, { keepFocus: true, noScroll: true });
	}

	async function onSearchFieldChange() {
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
		const id = metadataTarget.firkin.id;
		await firkinsService.enrich(id, {
			title: item.title,
			year: item.year,
			description: item.description ?? '',
			posterUrl: item.posterUrl,
			backdropUrl: item.backdropUrl
		});
		metadataTarget = null;
		// Refresh so the in-place-updated firkin replaces the cached one.
		await firkinsService.refresh();
	}

	onMount(() => {
		const stopFirkins = firkinsService.start();
		void (async () => {
			try {
				sources = await listSources();
			} catch (err) {
				sourcesError = err instanceof Error ? err.message : 'Unknown error';
			}
		})();
		return () => {
			stopFirkins();
		};
	});

	// Whenever the URL-driven `addon` changes (initial load, addon click, or
	// browser back/forward), reset search state and refetch genres.
	$effect(() => {
		const current = addon;
		if (!current) return;
		query = '';
		searchToken++;
		searchItems = [];
		searchTotalPages = 1;
		searchPage = 1;
		searchError = null;
		searchLoading = false;
		void refreshGenres();
	});
</script>

<svelte:head>
	<title>Mhaol Cloud — Catalog</title>
</svelte:head>

<section class="sticky top-0 z-50 border-b border-base-content/10 bg-base-200">
	<div class="grid grid-cols-2 gap-4 p-3">
		<div class="grid grid-cols-4 gap-2">
			{#each sources as source (source.id)}
				{@const active = addon === source.id}
				<button
					type="button"
					class={classNames('btn w-full btn-sm', {
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
		<div class="flex flex-wrap items-center gap-2">
			{#if showSearchFieldSelect}
				<select
					class="select-bordered select w-40 select-sm"
					bind:value={searchField}
					onchange={onSearchFieldChange}
					title="Which release-group field to search on"
				>
					<option value="artist">Artist name</option>
					<option value="release">Album title</option>
				</select>
			{/if}
			<input
				type="search"
				class="input-bordered input input-sm flex-1"
				placeholder={addon ? `Search ${currentSource?.label ?? addon}…` : 'Pick an addon to search'}
				disabled={!addon}
				bind:value={query}
				oninput={scheduleSearch}
			/>
		</div>
	</div>
	<div class="flex flex-wrap items-center gap-2 px-3 pb-3">
		<button
			type="button"
			class="btn btn-outline btn-sm"
			onclick={() => consumptionModalService.open()}
			title="Show playback time per firkin"
		>
			Consumption
		</button>
		<button
			type="button"
			class="btn btn-outline btn-sm"
			onclick={() => diskModalService.open()}
			title="Show host volumes and the data-root breakdown"
		>
			Disk
		</button>
		<button
			type="button"
			class="btn btn-outline btn-sm"
			onclick={() => librariesModalService.open()}
			title="Manage libraries on this machine"
		>
			Libraries
		</button>
		<button
			type="button"
			class="btn btn-outline btn-sm"
			onclick={() => ipfsModalService.open()}
			title="Show IPFS pins recorded by the cloud"
		>
			IPFS
		</button>
		<button
			type="button"
			class="btn btn-outline btn-sm"
			onclick={() => torrentModalService.open()}
			title="Show the embedded torrent client status"
		>
			Torrent
		</button>
		<button
			type="button"
			class="btn btn-outline btn-sm"
			onclick={() => artistsModalService.open()}
			title="Browse content-addressed artist records"
		>
			Artists
		</button>
		<button
			type="button"
			class="btn btn-outline btn-sm"
			onclick={() => healthModalService.open()}
			title="Live health of this Mhaol cloud node"
		>
			Health
		</button>
	</div>
</section>

<div class="flex flex-col gap-6 p-6">
	{#if sourcesError}
		<div class="alert alert-error">
			<span>Could not load catalog sources: {sourcesError}</span>
		</div>
	{/if}

	<LazyRow>
		<section class="flex flex-col gap-3">
			<div class="flex items-center justify-between gap-4">
				<h2 class="text-lg font-semibold">Library</h2>
				<label class="flex items-center gap-2 text-xs text-base-content/70">
					<input
						type="checkbox"
						class="toggle toggle-primary toggle-sm"
						checked={$firkinsIncludeAll}
						onchange={(e) =>
							firkinsService.setIncludeAll((e.currentTarget as HTMLInputElement).checked)}
					/>
					<span
						title="Off: only bookmarked firkins. On: every firkin in the local DB, including non-bookmarked browse-cache rows from the /catalog/visit resolver."
					>
						Show all locally-available
					</span>
				</label>
			</div>
			<FirkinLibraryGrid
				firkins={libraryAllFirkins}
				collapsed={true}
				collapsedCount={6}
				moreHref={galleryHref}
				emptyMessage={$firkinsIncludeAll
					? 'No firkins on this server yet.'
					: 'No bookmarked items yet — toggle "Show all locally-available" to include browse-cache items.'}
			>
				{#snippet actions(doc)}
					{#if firkinNeedsMetadata(doc) && metadataSearchAddon(doc.addon) !== null}
						<button
							type="button"
							class="btn absolute top-2 right-2 btn-xs btn-primary"
							onclick={() => openMetadataLookup(doc)}
							title="Search the relevant addon and bake matching metadata into this firkin"
						>
							Find metadata
						</button>
					{/if}
				{/snippet}
			</FirkinLibraryGrid>
		</section>
	</LazyRow>

	{#if addonRecommendationFirkins.length > 0}
		<LazyRow>
			<section class="flex flex-col gap-3">
				<div class="flex flex-wrap items-center justify-between gap-4">
					<h2 class="text-lg font-semibold">For you</h2>
				</div>
				<FirkinLibraryGrid
					firkins={addonRecommendationFirkins}
					collapsed={true}
					collapsedCount={6}
					moreHref={recommendationsHref}
					hrefBuilder={visitHrefForFirkin}
					emptyMessage="No recommendations for this addon yet."
				/>
			</section>
		</LazyRow>
	{/if}

	{#if hasSearch}
		<LazyRow>
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
								href={visitHref(item)}
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
		</LazyRow>
	{/if}

	{#if hasPopular}
		{#if hasFilter && genres.length > 0}
			{#each genres as genre (genre.id)}
				<LazyRow>
					<PopularGenreRow
						{addon}
						genreId={genre.id}
						title={genre.name}
						hrefBuilder={visitHrefForFirkin}
					/>
				</LazyRow>
			{/each}
		{:else}
			<LazyRow>
				<PopularGenreRow {addon} title="Popular" hrefBuilder={visitHrefForFirkin} />
			</LazyRow>
		{/if}
	{/if}
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
