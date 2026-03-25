<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { base } from '$app/paths';
	import { fetchRaw } from 'ui-lib/transport/fetch-helpers';
	import { workToDisplayDetails, authorToDisplay, getCoverUrl } from 'addons/openlibrary/transform';
	import type { DisplayBook, DisplayBookDetails } from 'addons/openlibrary/types';
	import type { OpenLibraryWork, OpenLibraryAuthor } from 'addons/openlibrary/types';
	import type { SmartSearchTorrentResult } from 'ui-lib/types/smart-search.type';
	import type { TorrentInfo } from 'ui-lib/types/torrent.type';
	import { smartSearchService } from 'ui-lib/services/smart-search.service';
	import { torrentService } from 'ui-lib/services/torrent.service';
	import BookDetailPage from 'ui-lib/components/books/BookDetailPage.svelte';

	let book = $state<DisplayBook | null>(null);
	let bookDetails = $state<DisplayBookDetails | null>(null);
	let loading = $state(true);
	let fetchingKey = $state<string | null>(null);

	const searchStore = smartSearchService.store;
	const torrentState = torrentService.state;

	let id = $derived($page.params.id ?? '');

	let isFetching = $derived(
		fetchingKey !== null &&
			fetchingKey === id &&
			$searchStore.fetchedCandidate === null &&
			$searchStore.selection?.mode === 'fetch'
	);
	let isFetchedForCurrent = $derived(
		$searchStore.fetchedCandidate !== null && fetchingKey === id
	);

	let currentFetchSteps = $derived.by(() => {
		if (!isFetching && !isFetchedForCurrent) return null;
		if (isFetchedForCurrent) {
			return { terms: true, search: true, searching: false, eval: true, done: true };
		}
		const s = $searchStore;
		const hasResults = s.searchResults.length > 0;
		const hasAnalysis = s.searchResults.some((r) => r.analysis !== null);
		return {
			terms: s.selection !== null,
			search: !s.searching && hasResults,
			searching: s.searching,
			eval: hasAnalysis,
			done: s.fetchedCandidate !== null
		};
	});

	let matchedTorrent = $derived.by((): TorrentInfo | null => {
		const candidate = $searchStore.fetchedCandidate;
		const _ = $torrentState;
		if (candidate?.infoHash) {
			const t = torrentService.findByHash(candidate.infoHash);
			if (t) return t;
		}
		return null;
	});

	let currentTorrentStatus = $derived(matchedTorrent);

	$effect(() => {
		const candidate = $searchStore.fetchedCandidate;
		const key = fetchingKey;
		if (candidate && key) {
			smartSearchService.saveBookFetchCache(key, candidate);
		}
	});

	async function fetchBook(bookKey: string) {
		loading = true;
		smartSearchService.clear();
		try {
			const workRes = await fetchRaw(`/api/openlibrary/works/${bookKey}`);
			if (!workRes.ok) throw new Error('Failed to fetch work');
			const work: OpenLibraryWork = await workRes.json();

			// Build display book from the work data
			const coverId = work.covers?.[0] ?? null;
			book = {
				key: bookKey,
				title: work.title,
				authors: work.authors?.map((a) => {
					const parts = a.author.key.split('/');
					return parts[parts.length - 1];
				}) ?? [],
				authorKeys: work.authors?.map((a) => a.author.key.replace('/authors/', '')) ?? [],
				firstPublishYear: work.first_publish_date?.split('-')[0] ?? '',
				coverId,
				coverUrl: getCoverUrl(coverId, 'M'),
				subjects: (work.subjects ?? []).slice(0, 10),
				publishers: [],
				pageCount: null,
				editionCount: 0,
				isbn: null,
				ratingsAverage: null,
				ratingsCount: 0
			};

			// Fetch author details
			const authorKeys = work.authors?.map((a) => a.author.key.replace('/authors/', '')) ?? [];
			const authors = await Promise.all(
				authorKeys.slice(0, 3).map(async (key) => {
					try {
						const res = await fetchRaw(`/api/openlibrary/authors/${key}`);
						if (res.ok) {
							const raw: OpenLibraryAuthor = await res.json();
							return authorToDisplay(raw);
						}
					} catch {
						// best-effort
					}
					return { key, name: 'Unknown', birthDate: null, deathDate: null, bio: null, photoUrl: null };
				})
			);
			bookDetails = workToDisplayDetails(work, authors, book);
			book = { ...book, authors: authors.map((a) => a.name) };

			// Check fetch cache
			const cached = await smartSearchService.checkBookFetchCache(bookKey);
			if (cached) {
				fetchingKey = bookKey;
				const sel = {
					title: book.title,
					year: book.firstPublishYear,
					type: 'book' as const,
					openlibraryKey: bookKey,
					author: book.authors[0] ?? 'Unknown',
					mode: 'fetch' as const
				};
				smartSearchService.setSelection(sel);
				smartSearchService.setFetchedCandidate(cached);
				smartSearchService.ensurePendingItem(sel);
			}
		} catch {
			book = null;
		}
		loading = false;
	}

	async function handleFetch() {
		if (!book) return;
		const isRefetch = isFetchedForCurrent;
		fetchingKey = book.key;
		if (!isRefetch) {
			const cached = await smartSearchService.checkBookFetchCache(book.key);
			if (cached) {
				const sel = {
					title: book.title,
					year: book.firstPublishYear,
					type: 'book' as const,
					openlibraryKey: book.key,
					author: book.authors[0] ?? 'Unknown',
					mode: 'fetch' as const
				};
				smartSearchService.setSelection(sel);
				smartSearchService.setFetchedCandidate(cached);
				smartSearchService.ensurePendingItem(sel);
				return;
			}
		}
		smartSearchService.select({
			title: book.title,
			year: book.firstPublishYear,
			type: 'book',
			openlibraryKey: book.key,
			author: book.authors[0] ?? 'Unknown',
			mode: 'fetch'
		});
	}

	function handleDownload() {
		const candidate = smartSearchService.getFetchedCandidate();
		if (!candidate) return;
		smartSearchService.startDownload(candidate);
	}

	onMount(() => {
		smartSearchService.initializeConfig();
		fetchBook(id);
	});
</script>

{#if book}
	<BookDetailPage
		{book}
		{bookDetails}
		{loading}
		fetching={isFetching}
		fetched={isFetchedForCurrent}
		fetchSteps={currentFetchSteps}
		torrentStatus={currentTorrentStatus}
		fetchedTorrent={$searchStore.fetchedCandidate
			? {
					name: $searchStore.fetchedCandidate.name,
					quality: $searchStore.fetchedCandidate.analysis?.quality ?? '',
					languages: $searchStore.fetchedCandidate.analysis?.languages ?? ''
				}
			: null}
		onfetch={handleFetch}
		ondownload={handleDownload}
		onshowsearch={() => smartSearchService.show()}
		onback={() => goto(`${base}/books`)}
	/>
{:else if loading}
	<div class="flex flex-1 items-center justify-center">
		<span class="loading loading-lg loading-spinner"></span>
	</div>
{:else}
	<div class="flex flex-1 flex-col items-center justify-center gap-2">
		<p class="text-sm opacity-60">Book not found</p>
		<button class="btn btn-ghost btn-sm" onclick={() => goto(`${base}/books`)}>Back to books</button>
	</div>
{/if}
