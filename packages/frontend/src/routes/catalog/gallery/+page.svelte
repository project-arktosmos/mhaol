<script lang="ts">
	import { onMount } from 'svelte';
	import { base } from '$app/paths';
	import { page as pageStore } from '$app/state';
	import FirkinLibraryGrid from '$components/catalog/FirkinLibraryGrid.svelte';
	import FirkinMetadataLookupModal, {
		type CatalogLookupItem
	} from '$components/firkins/FirkinMetadataLookupModal.svelte';
	import {
		listSources,
		loadGenres,
		loadPopular,
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
	import type { CloudFirkin } from '$types/firkin.type';

	const firkinsStore = firkinsService.state;
	const firkinsIncludeAll = firkinsService.includeAll;
	const userIdentityState = userIdentityService.state;

	type GalleryMode = 'library' | 'popular' | 'for-you';

	const LOCAL_ADDON_FOR: Record<string, string> = {
		'tmdb-movie': 'local-movie',
		'tmdb-tv': 'local-tv',
		musicbrainz: 'local-album'
	};

	let sources = $state<CatalogSource[]>([]);
	let sourcesError = $state<string | null>(null);

	const addon = $derived(pageStore.url.searchParams.get('addon') ?? '');
	const modeParam = $derived(pageStore.url.searchParams.get('mode') ?? 'library');
	const mode = $derived<GalleryMode>(
		modeParam === 'popular' || modeParam === 'for-you' ? modeParam : 'library'
	);
	const filterParam = $derived(pageStore.url.searchParams.get('filter') ?? '');
	const currentSource = $derived(sources.find((s) => s.id === addon));
	const sourceLabel = $derived(currentSource?.label ?? addon);

	let genres = $state<CatalogGenre[]>([]);
	let lastGenresAddon: string | null = null;

	$effect(() => {
		if (mode !== 'popular') return;
		const supportsFilter = currentSource?.hasFilter ?? false;
		if (!addon || !supportsFilter) {
			genres = [];
			lastGenresAddon = null;
			return;
		}
		if (lastGenresAddon === addon) return;
		lastGenresAddon = addon;
		void (async () => {
			try {
				genres = await loadGenres(addon);
			} catch {
				genres = [];
			}
		})();
	});

	const filterLabel = $derived(
		filterParam ? (genres.find((g) => g.id === filterParam)?.name ?? filterParam) : ''
	);
	const popularHeadingSuffix = $derived(filterLabel ? ` · ${filterLabel}` : '');
	const headingLabel = $derived(
		mode === 'popular'
			? sourceLabel
				? `Popular ${sourceLabel}${popularHeadingSuffix}`
				: `Popular${popularHeadingSuffix}`
			: mode === 'for-you'
				? sourceLabel
					? `For you · ${sourceLabel}`
					: 'For you'
				: sourceLabel
					? `${sourceLabel} library`
					: 'Library gallery'
	);

	// --- Library mode --------------------------------------------------------

	const galleryFirkins = $derived<Firkin[]>(
		mode === 'library' && addon
			? $firkinsStore.firkins
					.filter((d) => d.addon === addon || d.addon === LOCAL_ADDON_FOR[addon])
					.slice()
					.sort((a, b) => b.created_at.localeCompare(a.created_at))
			: []
	);

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
		await firkinsService.refresh();
	}

	// --- Popular mode --------------------------------------------------------

	let popularItems = $state<CatalogItem[]>([]);
	let popularPage = $state(1);
	let popularTotalPages = $state(1);
	let popularLoading = $state(false);
	let popularError = $state<string | null>(null);

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

	const popularFirkins = $derived<CloudFirkin[]>(
		mode === 'popular' ? popularItems.map((it) => virtualFirkin(it)) : []
	);

	async function refreshPopular() {
		if (mode !== 'popular' || !addon) {
			popularItems = [];
			popularTotalPages = 1;
			return;
		}
		popularLoading = true;
		popularError = null;
		try {
			const result = await loadPopular(addon, {
				page: popularPage,
				filter: filterParam || undefined
			});
			popularItems = result.items;
			popularTotalPages = result.totalPages;
			popularPage = result.page;
		} catch (err) {
			popularItems = [];
			popularError = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			popularLoading = false;
		}
	}

	async function goToPopularPage(next: number) {
		if (next < 1 || next > popularTotalPages || next === popularPage) return;
		popularPage = next;
		await refreshPopular();
	}

	// --- For-you mode --------------------------------------------------------

	let recommendations = $state<Recommendation[]>([]);
	let lastRecommendationsAddress: string | null = null;

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

	const forYouFirkins = $derived<CloudFirkin[]>(
		mode === 'for-you' && addon
			? recommendations
					.filter((r) => r.addon === addon)
					.map((r) => recommendationToFirkin(r))
			: []
	);

	$effect(() => {
		if (mode !== 'for-you') return;
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

	// --- Shared helpers ------------------------------------------------------

	function visitHrefForFirkin(firkin: CloudFirkin): string {
		const prefix = `virtual:${firkin.addon}:`;
		const upstreamId = firkin.id.startsWith(prefix)
			? firkin.id.slice(prefix.length)
			: firkin.id;
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

	$effect(() => {
		if (mode !== 'popular') return;
		void addon;
		void filterParam;
		popularPage = 1;
		void refreshPopular();
	});
</script>

<svelte:head>
	<title>Mhaol Cloud — {headingLabel}</title>
</svelte:head>

<div class="flex min-h-full flex-col gap-4 p-6">
	<header class="flex flex-wrap items-center justify-between gap-3">
		<div class="flex items-center gap-3">
			<a
				href={addon ? `${base}/catalog?addon=${encodeURIComponent(addon)}` : `${base}/catalog`}
				class="btn btn-ghost btn-sm"
			>
				← Back to catalog
			</a>
			<h1 class="text-xl font-semibold">{headingLabel}</h1>
			{#if mode === 'library'}
				<span class="badge badge-ghost">{galleryFirkins.length}</span>
			{:else if mode === 'for-you'}
				<span class="badge badge-ghost">{forYouFirkins.length}</span>
			{/if}
		</div>
		{#if mode === 'library'}
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
		{:else if mode === 'popular'}
			<div class="flex items-center gap-2">
				<button
					class="btn btn-outline btn-xs"
					onclick={() => goToPopularPage(popularPage - 1)}
					disabled={popularLoading || popularPage <= 1}
				>
					Prev
				</button>
				<span class="text-xs text-base-content/60">
					Page {popularPage} / {popularTotalPages}
				</span>
				<button
					class="btn btn-outline btn-xs"
					onclick={() => goToPopularPage(popularPage + 1)}
					disabled={popularLoading || popularPage >= popularTotalPages}
				>
					Next
				</button>
			</div>
		{/if}
	</header>

	{#if sourcesError}
		<div class="alert alert-warning">
			<span>Could not load catalog sources: {sourcesError}</span>
		</div>
	{/if}

	{#if !addon}
		<p class="text-sm text-base-content/60">
			No addon selected — open the gallery from the
			<a class="link" href={`${base}/catalog`}>catalog page</a>.
		</p>
	{:else if mode === 'library'}
		<FirkinLibraryGrid
			firkins={galleryFirkins}
			collapsed={false}
			emptyMessage={$firkinsIncludeAll
				? `No firkins for ${sourceLabel} yet.`
				: `No bookmarked ${sourceLabel} items yet — toggle "Show all locally-available" to include browse-cache items.`}
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
	{:else if mode === 'popular'}
		{#if popularError}
			<div class="alert alert-error"><span>{popularError}</span></div>
		{/if}
		{#if popularLoading && popularItems.length === 0}
			<p class="text-sm text-base-content/60">Loading…</p>
		{:else}
			<div class={popularLoading ? 'opacity-60' : ''}>
				<FirkinLibraryGrid
					firkins={popularFirkins}
					collapsed={false}
					hrefBuilder={visitHrefForFirkin}
					emptyMessage={`No popular ${sourceLabel} items.`}
				/>
			</div>
		{/if}
	{:else if mode === 'for-you'}
		{#if !$userIdentityState.identity}
			<div class="alert alert-warning">
				<span>Sign in on the Profile page to see your recommendations.</span>
			</div>
		{:else}
			<FirkinLibraryGrid
				firkins={forYouFirkins}
				collapsed={false}
				hrefBuilder={visitHrefForFirkin}
				emptyMessage={`No recommendations for ${sourceLabel} yet.`}
			/>
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
