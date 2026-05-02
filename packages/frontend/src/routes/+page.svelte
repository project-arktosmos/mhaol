<script lang="ts">
	import { onMount } from 'svelte';
	import { base } from '$app/paths';
	import { page as pageStore } from '$app/state';
	import FirkinLibraryGrid from '$components/catalog/FirkinLibraryGrid.svelte';
	import LazyRow from '$components/catalog/LazyRow.svelte';
	import PopularGenreRow from '$components/catalog/PopularGenreRow.svelte';
	import NavbarAddonPicker from '$components/core/NavbarAddonPicker.svelte';
	import NavbarSearch from '$components/core/NavbarSearch.svelte';
	import FirkinMetadataLookupModal, {
		type CatalogLookupItem
	} from '$components/firkins/FirkinMetadataLookupModal.svelte';
	import type { CloudFirkin } from '$types/firkin.type';
	import {
		listSources,
		loadGenres,
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
	import { listMediaTrackers, type MediaTracker } from '$lib/media-tracker.service';
	import { userIdentityService } from '$lib/user-identity.service';
	import { addonKind, type FirkinKind } from 'cloud-ui';

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

	let genres = $state<CatalogGenre[]>([]);

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

	let trackers = $state<MediaTracker[]>([]);
	let lastTrackersAddress: string | null = null;

	$effect(() => {
		const address = $userIdentityState.identity?.address;
		if (!address) {
			trackers = [];
			lastTrackersAddress = null;
			return;
		}
		if (lastTrackersAddress === address) return;
		lastTrackersAddress = address;
		void (async () => {
			try {
				trackers = await listMediaTrackers(address);
			} catch {
				trackers = [];
			}
		})();
	});

	// Per-kind playback duration in seconds — used as the denominator for the
	// progress bar on the Continue row. We don't persist real runtimes on the
	// firkin record, so these are sensible upper bounds: the bar caps at 100%
	// once accumulated playtime crosses the typical full-watch duration.
	const KIND_FULL_SECONDS: Record<FirkinKind, number> = {
		movie: 7200,
		'tv show': 2700,
		album: 2700,
		'youtube video': 600,
		book: 3600,
		game: 3600
	};

	const continueProgressById = $derived.by<Record<string, number>>(() => {
		const map: Record<string, number> = {};
		const totals = new Map<string, number>();
		for (const row of trackers) {
			totals.set(row.firkinId, (totals.get(row.firkinId) ?? 0) + row.totalSeconds);
		}
		for (const f of $firkinsStore.firkins) {
			const total = totals.get(f.id);
			if (!total || total <= 0) continue;
			const kind = addonKind(f.addon) ?? 'movie';
			const denom = KIND_FULL_SECONDS[kind] ?? 7200;
			map[f.id] = Math.min(0.99, total / denom);
		}
		return map;
	});

	const continueFirkins = $derived<Firkin[]>(
		addon
			? $firkinsStore.firkins
					.filter((f) => f.addon === addon || f.addon === LOCAL_ADDON_FOR[addon])
					.filter((f) => (continueProgressById[f.id] ?? 0) > 0)
					.slice()
					.sort((a, b) => {
						const ta = trackers.find((t) => t.firkinId === a.id)?.last_played_at ?? '';
						const tb = trackers.find((t) => t.firkinId === b.id)?.last_played_at ?? '';
						return tb.localeCompare(ta);
					})
			: []
	);

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

	// Refetch genres whenever the URL-driven `addon` changes.
	$effect(() => {
		if (!addon) return;
		void refreshGenres();
	});
</script>

<svelte:head>
	<title>Mhaol Cloud — Catalog</title>
</svelte:head>

<section class="sticky top-0 z-50 border-b border-base-content/10 bg-base-200">
	<div class="grid w-full grid-cols-2 items-stretch gap-3 p-3">
		<NavbarAddonPicker classes="grid grid-cols-4 gap-1" />
		<NavbarSearch
			classes="flex items-stretch gap-2 w-full"
			inputClasses="input-bordered input input-sm flex-1 min-w-0 h-full"
		/>
	</div>
</section>

<div class="flex flex-col gap-6 p-6">
	{#if sourcesError}
		<div class="alert alert-error">
			<span>Could not load catalog sources: {sourcesError}</span>
		</div>
	{/if}

	{#if continueFirkins.length > 0}
		<LazyRow>
			<section class="flex flex-col gap-3">
				<div class="flex flex-wrap items-center justify-between gap-4">
					<h2 class="text-lg font-semibold">Continue</h2>
				</div>
				<FirkinLibraryGrid
					firkins={continueFirkins}
					collapsed={true}
					collapsedCount={6}
					progressFor={(f) => continueProgressById[f.id] ?? null}
					emptyMessage="Nothing in progress yet."
				/>
			</section>
		</LazyRow>
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
