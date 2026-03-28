<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { base } from '$app/paths';
	import { error } from '@sveltejs/kit';
	import { fetchRaw } from 'ui-lib/transport/fetch-helpers';
	import { smartSearchService } from 'ui-lib/services/smart-search.service';
	import { torrentService } from 'ui-lib/services/torrent.service';
	import { favoritesService } from 'ui-lib/services/favorites.service';
	import { pinsService } from 'ui-lib/services/pins.service';
	import { getMediaConfig } from 'ui-lib/data/media-registry';
	import type { MediaTypeConfig } from 'ui-lib/data/media-registry';
	import type { CatalogItem, CatalogBook, CatalogGame } from 'ui-lib/types/catalog.type';
	import type { IptvChannel, IptvStream, IptvEpgProgram } from 'ui-lib/types/iptv.type';

	// Detail components
	import CatalogDetailPage from 'ui-lib/components/catalog/CatalogDetailPage.svelte';
	import BookDetailMeta from 'ui-lib/components/catalog/detail/BookDetailMeta.svelte';
	import GameDetailMeta from 'ui-lib/components/catalog/detail/GameDetailMeta.svelte';
	import IptvChannelDetail from 'ui-lib/components/iptv/IptvChannelDetail.svelte';

	// Type-specific transforms
	import { workToDisplayDetails, authorToDisplay, getCoverUrl } from 'addons/openlibrary/transform';
	import type { OpenLibraryWork, OpenLibraryAuthor } from 'addons/openlibrary/types';
	import { gameExtendedToDisplay } from 'addons/retroachievements';
	import type { RaGameExtended } from 'addons/retroachievements/types';
	import { iptvService } from 'ui-lib/services/iptv.service';

	let slug = $derived($page.params.slug ?? '');
	let id = $derived($page.params.id ?? '');
	let config = $derived(getMediaConfig(slug));

	// Shared state
	let catalogItem = $state<CatalogItem | null>(null);
	let loading = $state(true);
	let fetchingId = $state<string | null>(null);

	const searchStore = smartSearchService.store;
	const favState = favoritesService.state;
	const pinState = pinsService.state;

	let isFavorite = $derived(
		config ? $favState.items.some((f) => f.service === config.favService && f.serviceId === id) : false
	);
	let isPinned = $derived(
		config ? $pinState.items.some((p) => p.service === config.pinService && p.serviceId === id) : false
	);

	// Smart search state (shared across catalog types)
	let isFetching = $derived(
		fetchingId !== null && fetchingId === id &&
			$searchStore.fetchedCandidate === null && $searchStore.selection?.mode === 'fetch'
	);
	let isFetchedForCurrent = $derived(
		$searchStore.fetchedCandidate !== null && fetchingId === id
	);
	let currentFetchSteps = $derived.by(() => {
		if (!isFetching && !isFetchedForCurrent) return null;
		if (isFetchedForCurrent) return { terms: true, search: true, searching: false, eval: true, done: true };
		const s = $searchStore;
		return {
			terms: s.selection !== null,
			search: !s.searching && s.searchResults.length > 0,
			searching: s.searching,
			eval: s.searchResults.some((r) => r.analysis !== null),
			done: s.fetchedCandidate !== null
		};
	});
	let matchedTorrent = $derived.by(() => {
		const candidate = $searchStore.fetchedCandidate;
		if (candidate?.infoHash) {
			const t = torrentService.findByHash(candidate.infoHash);
			if (t) return t;
		}
		return null;
	});

	// IPTV-specific state
	let iptvChannel = $state<IptvChannel | null>(null);
	let iptvStreams = $state<IptvStream[]>([]);
	let iptvStreamUrl = $state('');
	let iptvEpgPrograms = $state<IptvEpgProgram[]>([]);
	let iptvEpgAvailable = $state(false);
	let togglingFavorite = $state(false);
	let togglingPin = $state(false);

	// === Per-type fetch functions ===

	async function fetchBook(bookKey: string) {
		loading = true;
		smartSearchService.clear();
		try {
			const workRes = await fetchRaw(`/api/openlibrary/works/${bookKey}`);
			if (!workRes.ok) throw new Error('Failed to fetch');
			const work: OpenLibraryWork = await workRes.json();
			const coverId = work.covers?.[0] ?? null;
			const authorKeys = work.authors?.map((a) => a.author.key.replace('/authors/', '')) ?? [];
			const authors = await Promise.all(
				authorKeys.slice(0, 3).map(async (key) => {
					try {
						const res = await fetchRaw(`/api/openlibrary/authors/${key}`);
						if (res.ok) return authorToDisplay(await res.json() as OpenLibraryAuthor);
					} catch { /* best-effort */ }
					return { key, name: 'Unknown', birthDate: null, deathDate: null, bio: null, photoUrl: null };
				})
			);
			const details = workToDisplayDetails(work, authors, {
				key: bookKey, title: work.title, authors: authorKeys, authorKeys,
				firstPublishYear: work.first_publish_date?.split('-')[0] ?? '',
				coverId, coverUrl: getCoverUrl(coverId, 'M'),
				subjects: (work.subjects ?? []).slice(0, 10), publishers: [],
				pageCount: null, editionCount: 0, isbn: null, ratingsAverage: null, ratingsCount: 0
			});
			catalogItem = {
				id: bookKey, kind: 'book',
				title: work.title, sortTitle: work.title.toLowerCase(),
				year: work.first_publish_date?.split('-')[0] ?? null,
				overview: details?.description ?? null,
				posterUrl: getCoverUrl(coverId, 'M'), backdropUrl: null,
				voteAverage: null, voteCount: null,
				parentId: null, position: null,
				source: 'openlibrary', sourceId: bookKey,
				createdAt: '', updatedAt: '',
				metadata: {
					openlibraryKey: bookKey, authors: authors.map((a) => a.name), authorKeys,
					firstPublishYear: work.first_publish_date?.split('-')[0] ?? '',
					coverId, coverUrl: getCoverUrl(coverId, 'M'),
					subjects: (work.subjects ?? []).slice(0, 10), publishers: [],
					pageCount: null, editionCount: 0, isbn: null,
					ratingsAverage: null, ratingsCount: 0,
					description: details?.description ?? null, authorDetails: authors
				}
			};
			const cached = await smartSearchService.checkBookFetchCache(bookKey);
			if (cached) {
				fetchingId = bookKey;
				const sel = {
					title: work.title, year: catalogItem.year ?? '',
					type: 'book' as const, openlibraryKey: bookKey,
					author: authors[0]?.name ?? 'Unknown', mode: 'fetch' as const
				};
				smartSearchService.setSelection(sel);
				smartSearchService.setFetchedCandidate(cached);
				smartSearchService.ensurePendingItem(sel);
			}
		} catch { catalogItem = null; }
		loading = false;
	}

	async function fetchGame(gameId: string) {
		loading = true;
		smartSearchService.clear();
		try {
			const res = await fetchRaw(`/api/retroachievements/games/${gameId}`);
			if (!res.ok) throw new Error('Failed to fetch');
			const detail = gameExtendedToDisplay(await res.json() as RaGameExtended);
			catalogItem = {
				id: String(detail.id), kind: 'game',
				title: detail.title, sortTitle: detail.title.toLowerCase(),
				year: detail.released ?? null, overview: null,
				posterUrl: detail.imageBoxArtUrl ?? detail.imageIconUrl, backdropUrl: detail.imageTitleUrl ?? null,
				voteAverage: null, voteCount: null,
				parentId: null, position: null,
				source: 'retroachievements', sourceId: String(detail.id),
				createdAt: '', updatedAt: '',
				metadata: {
					retroachievementsId: detail.id, consoleId: detail.consoleId,
					consoleName: detail.consoleName, imageIconUrl: detail.imageIconUrl,
					numAchievements: detail.numAchievements, points: detail.points,
					developer: detail.developer ?? null, publisher: detail.publisher ?? null,
					genre: detail.genre ?? null, released: detail.released ?? null,
					imageTitleUrl: detail.imageTitleUrl ?? null,
					imageIngameUrl: detail.imageIngameUrl ?? null,
					imageBoxArtUrl: detail.imageBoxArtUrl ?? null,
					achievements: (detail.achievements ?? []).map((a) => ({
						id: a.id, title: a.title, description: a.description,
						points: a.points, trueRatio: a.trueRatio, badgeUrl: a.badgeUrl,
						displayOrder: a.displayOrder, numAwarded: a.numAwarded,
						numAwardedHardcore: a.numAwardedHardcore
					}))
				}
			};
		} catch { catalogItem = null; }
		loading = false;
	}

	async function fetchIptv(channelId: string) {
		loading = true;
		const detail = await iptvService.getChannel(channelId);
		if (detail) {
			iptvChannel = detail.channel;
			iptvStreams = detail.streams;
			if (detail.streams.length > 0) {
				iptvStreamUrl = iptvService.getStreamUrl(channelId);
			}
		}
		loading = false;
		const epg = await iptvService.getEpg(channelId);
		if (epg) {
			iptvEpgAvailable = epg.available;
			iptvEpgPrograms = epg.programs;
		}
	}

	// === Shared smart search handlers ===

	function handleFetch() {
		if (!catalogItem || !config) return;
		fetchingId = catalogItem.sourceId;
		if (catalogItem.kind === 'book') {
			const book = catalogItem as CatalogBook;
			smartSearchService.select({
				title: book.title, year: book.year ?? '', type: 'book',
				openlibraryKey: book.sourceId,
				author: book.metadata.authors[0] ?? 'Unknown', mode: 'fetch'
			});
		} else if (catalogItem.kind === 'game') {
			const game = catalogItem as CatalogGame;
			smartSearchService.select({
				title: game.title, year: '', type: 'game',
				retroachievementsId: game.metadata.retroachievementsId,
				consoleName: game.metadata.consoleName, mode: 'fetch'
			});
		}
	}

	function handleDownload() {
		const candidate = smartSearchService.getFetchedCandidate();
		if (candidate) smartSearchService.startDownload(candidate);
	}

	async function handleToggleFavorite() {
		if (!config) return;
		if (config.kind === 'iptv_channel') {
			if (!iptvChannel) return;
			togglingFavorite = true;
			try { await favoritesService.toggle(config.favService, iptvChannel.id, iptvChannel.name); }
			finally { togglingFavorite = false; }
		} else if (catalogItem) {
			await favoritesService.toggle(config.favService, catalogItem.sourceId, catalogItem.title);
		}
	}

	async function handleTogglePin() {
		if (!config) return;
		if (config.kind === 'iptv_channel') {
			if (!iptvChannel) return;
			togglingPin = true;
			try { await pinsService.toggle(config.pinService, iptvChannel.id, iptvChannel.name); }
			finally { togglingPin = false; }
		} else if (catalogItem) {
			await pinsService.toggle(config.pinService, catalogItem.sourceId, catalogItem.title);
		}
	}

	// Book fetch cache auto-save
	$effect(() => {
		const candidate = $searchStore.fetchedCandidate;
		if (candidate && fetchingId && config?.kind === 'book') {
			smartSearchService.saveBookFetchCache(fetchingId, candidate);
		}
	});

	onMount(() => {
		if (!config) return;
		smartSearchService.initializeConfig();
		if (config.kind === 'book') fetchBook(id);
		else if (config.kind === 'game') fetchGame(id);
		else if (config.kind === 'iptv_channel') fetchIptv(id);
	});
</script>

{#if !config}
	<div class="flex flex-1 items-center justify-center">
		<p class="text-sm opacity-60">Not found</p>
	</div>
{:else if config.kind === 'iptv_channel'}
	{#if iptvChannel}
		<IptvChannelDetail
			channel={iptvChannel}
			streams={iptvStreams}
			streamUrl={iptvStreamUrl}
			{loading}
			epgPrograms={iptvEpgPrograms}
			epgAvailable={iptvEpgAvailable}
			{isFavorite}
			{togglingFavorite}
			{isPinned}
			{togglingPin}
			onback={() => goto(`${base}/media/iptv`)}
			onstreamselect={() => { iptvStreamUrl = iptvService.getStreamUrl(iptvChannel?.id ?? id); }}
			ontogglefavorite={handleToggleFavorite}
			ontogglepin={handleTogglePin}
		/>
	{:else if loading}
		<div class="flex flex-1 items-center justify-center"><span class="loading loading-lg loading-spinner"></span></div>
	{:else}
		<div class="flex flex-1 flex-col items-center justify-center gap-2">
			<p class="text-sm opacity-60">Channel not found</p>
			<button class="btn btn-ghost btn-sm" onclick={() => goto(`${base}/media/iptv`)}>Back</button>
		</div>
	{/if}
{:else if catalogItem}
	<CatalogDetailPage
		item={catalogItem} {loading}
		fetching={isFetching} fetched={isFetchedForCurrent}
		fetchSteps={currentFetchSteps} torrentStatus={matchedTorrent}
		fetchedTorrent={$searchStore.fetchedCandidate ? { name: $searchStore.fetchedCandidate.name, quality: $searchStore.fetchedCandidate.analysis?.quality ?? '', languages: $searchStore.fetchedCandidate.analysis?.languages ?? '' } : null}
		{isFavorite} {isPinned}
		onfetch={handleFetch} ondownload={handleDownload}
		onshowsearch={() => smartSearchService.show()}
		onback={() => goto(`${base}/media/${config.slug}`)}
		ontogglefavorite={handleToggleFavorite} ontogglepin={handleTogglePin}
	>
		{#snippet extra()}
			{#if catalogItem?.kind === 'book'}
				<BookDetailMeta item={catalogItem} />
			{:else if catalogItem?.kind === 'game'}
				<GameDetailMeta item={catalogItem} />
			{/if}
		{/snippet}
	</CatalogDetailPage>
{:else if loading}
	<div class="flex flex-1 items-center justify-center"><span class="loading loading-lg loading-spinner"></span></div>
{:else}
	<div class="flex flex-1 flex-col items-center justify-center gap-2">
		<p class="text-sm opacity-60">Not found</p>
		<button class="btn btn-ghost btn-sm" onclick={() => goto(`${base}/media/${config.slug}`)}>Back</button>
	</div>
{/if}
