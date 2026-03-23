<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { apiUrl } from 'ui-lib/lib/api-base';
	import { workToDisplayDetails, authorToDisplay, getCoverUrl } from 'addons/openlibrary/transform';
	import type { DisplayBook, DisplayBookDetails } from 'addons/openlibrary/types';
	import type { OpenLibraryWork, OpenLibraryAuthor } from 'addons/openlibrary/types';
	import { smartSearchService } from 'ui-lib/services/smart-search.service';
	import BookDetailPage from 'ui-lib/components/books/BookDetailPage.svelte';

	let book = $state<DisplayBook | null>(null);
	let bookDetails = $state<DisplayBookDetails | null>(null);
	let loading = $state(true);

	const smartSearchState = smartSearchService.store;

	let fetching = $derived(
		$smartSearchState.selection?.type === 'book' &&
			($smartSearchState.searching || $smartSearchState.analyzing)
	);
	let fetched = $derived(
		$smartSearchState.selection?.type === 'book' &&
			book !== null &&
			$smartSearchState.selection.openlibraryKey === book.key &&
			!!$smartSearchState.fetchedCandidate
	);
	let fetchSteps = $derived.by(() => {
		if ($smartSearchState.selection?.type !== 'book') return null;
		if (!$smartSearchState.selection && !$smartSearchState.fetchedCandidate) return null;
		return {
			terms: !!$smartSearchState.selection,
			search: $smartSearchState.searchResults.length > 0,
			searching: $smartSearchState.searching,
			eval: $smartSearchState.analyzing,
			done: !!$smartSearchState.fetchedCandidate
		};
	});

	let id = $derived($page.params.id ?? '');

	async function fetchBook(bookKey: string) {
		loading = true;
		try {
			const workRes = await fetch(apiUrl(`/api/openlibrary/works/${bookKey}`));
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
						const res = await fetch(apiUrl(`/api/openlibrary/authors/${key}`));
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
			// Update author names on the display book
			book = { ...book, authors: authors.map((a) => a.name) };
		} catch {
			book = null;
		}
		loading = false;
	}

	function handleFetch() {
		if (!book) return;
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
		{fetching}
		{fetched}
		{fetchSteps}
		onfetch={handleFetch}
		ondownload={handleDownload}
		onback={() => goto('/books')}
	/>
{:else if loading}
	<div class="flex flex-1 items-center justify-center">
		<span class="loading loading-lg loading-spinner"></span>
	</div>
{:else}
	<div class="flex flex-1 flex-col items-center justify-center gap-2">
		<p class="text-sm opacity-60">Book not found</p>
		<button class="btn btn-ghost btn-sm" onclick={() => goto('/books')}>Back to books</button>
	</div>
{/if}
