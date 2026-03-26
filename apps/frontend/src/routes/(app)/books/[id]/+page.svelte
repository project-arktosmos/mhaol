<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { base } from '$app/paths';
	import { fetchRaw } from 'ui-lib/transport/fetch-helpers';
	import { workToDisplayDetails, authorToDisplay, getCoverUrl } from 'addons/openlibrary/transform';
	import type { OpenLibraryWork, OpenLibraryAuthor } from 'addons/openlibrary/types';
	import { smartSearchService } from 'ui-lib/services/smart-search.service';
	import { torrentService } from 'ui-lib/services/torrent.service';
	import { favoritesService } from 'ui-lib/services/favorites.service';
	import { pinsService } from 'ui-lib/services/pins.service';
	import type { CatalogBook } from 'ui-lib/types/catalog.type';
	import CatalogDetailPage from 'ui-lib/components/catalog/CatalogDetailPage.svelte';
	import BookDetailMeta from 'ui-lib/components/catalog/detail/BookDetailMeta.svelte';

	let catalogItem = $state<CatalogBook | null>(null);
	let loading = $state(true);
	let fetchingKey = $state<string | null>(null);

	const favState = favoritesService.state;
	const pinState = pinsService.state;
	const searchStore = smartSearchService.store;

	let id = $derived($page.params.id ?? '');

	let isFavorite = $derived(
		$favState.items.some((f) => f.service === 'openlibrary' && f.serviceId === id)
	);
	let isPinned = $derived(
		$pinState.items.some((p) => p.service === 'openlibrary' && p.serviceId === id)
	);

	let isFetching = $derived(
		fetchingKey !== null && fetchingKey === id &&
			$searchStore.fetchedCandidate === null && $searchStore.selection?.mode === 'fetch'
	);
	let isFetchedForCurrent = $derived(
		$searchStore.fetchedCandidate !== null && fetchingKey === id
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

	$effect(() => {
		const candidate = $searchStore.fetchedCandidate;
		if (candidate && fetchingKey) smartSearchService.saveBookFetchCache(fetchingKey, candidate);
	});

	async function fetchBook(bookKey: string) {
		loading = true;
		smartSearchService.clear();
		try {
			const workRes = await fetchRaw(`/api/openlibrary/works/${bookKey}`);
			if (!workRes.ok) throw new Error('Failed to fetch work');
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
					openlibraryKey: bookKey,
					authors: authors.map((a) => a.name),
					authorKeys,
					firstPublishYear: work.first_publish_date?.split('-')[0] ?? '',
					coverId, coverUrl: getCoverUrl(coverId, 'M'),
					subjects: (work.subjects ?? []).slice(0, 10),
					publishers: [], pageCount: null, editionCount: 0,
					isbn: null, ratingsAverage: null, ratingsCount: 0,
					description: details?.description ?? null,
					authorDetails: authors
				}
			};

			const cached = await smartSearchService.checkBookFetchCache(bookKey);
			if (cached) {
				fetchingKey = bookKey;
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

	async function handleFetch() {
		if (!catalogItem) return;
		fetchingKey = catalogItem.sourceId;
		if (!isFetchedForCurrent) {
			const cached = await smartSearchService.checkBookFetchCache(catalogItem.sourceId);
			if (cached) {
				const sel = {
					title: catalogItem.title, year: catalogItem.year ?? '',
					type: 'book' as const, openlibraryKey: catalogItem.sourceId,
					author: catalogItem.metadata.authors[0] ?? 'Unknown', mode: 'fetch' as const
				};
				smartSearchService.setSelection(sel);
				smartSearchService.setFetchedCandidate(cached);
				smartSearchService.ensurePendingItem(sel);
				return;
			}
		}
		smartSearchService.select({
			title: catalogItem.title, year: catalogItem.year ?? '', type: 'book',
			openlibraryKey: catalogItem.sourceId,
			author: catalogItem.metadata.authors[0] ?? 'Unknown', mode: 'fetch'
		});
	}

	function handleDownload() {
		const candidate = smartSearchService.getFetchedCandidate();
		if (candidate) smartSearchService.startDownload(candidate);
	}

	async function handleToggleFavorite() {
		if (catalogItem) await favoritesService.toggle('openlibrary', catalogItem.sourceId, catalogItem.title);
	}

	async function handleTogglePin() {
		if (catalogItem) await pinsService.toggle('openlibrary', catalogItem.sourceId, catalogItem.title);
	}

	onMount(() => { smartSearchService.initializeConfig(); fetchBook(id); });
</script>

{#if catalogItem}
	<CatalogDetailPage
		item={catalogItem} {loading}
		fetching={isFetching} fetched={isFetchedForCurrent}
		fetchSteps={currentFetchSteps} torrentStatus={matchedTorrent}
		fetchedTorrent={$searchStore.fetchedCandidate ? { name: $searchStore.fetchedCandidate.name, quality: $searchStore.fetchedCandidate.analysis?.quality ?? '', languages: $searchStore.fetchedCandidate.analysis?.languages ?? '' } : null}
		{isFavorite}
		{isPinned}
		onfetch={handleFetch} ondownload={handleDownload}
		onshowsearch={() => smartSearchService.show()}
		onback={() => goto(`${base}/books`)}
		ontogglefavorite={handleToggleFavorite}
		ontogglepin={handleTogglePin}
	>
		{#snippet extra()}
			<BookDetailMeta item={catalogItem!} />
		{/snippet}
	</CatalogDetailPage>
{:else if loading}
	<div class="flex flex-1 items-center justify-center"><span class="loading loading-lg loading-spinner"></span></div>
{:else}
	<div class="flex flex-1 flex-col items-center justify-center gap-2">
		<p class="text-sm opacity-60">Book not found</p>
		<button class="btn btn-ghost btn-sm" onclick={() => goto(`${base}/books`)}>Back to books</button>
	</div>
{/if}
