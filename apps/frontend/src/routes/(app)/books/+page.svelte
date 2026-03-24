<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { bookBrowseService } from 'ui-lib/services/book-browse.service';
	import { BOOK_SUBJECTS, BOOK_SUBJECT_LABELS } from 'addons/openlibrary/types';
	import type { DisplayBook, BookSubject } from 'addons/openlibrary/types';
	import BookBrowseGrid from 'ui-lib/components/books/BookBrowseGrid.svelte';
	import classNames from 'classnames';

	const browseState = bookBrowseService.state;

	let activeTab = $state<'search' | 'trending'>('trending');
	let searchInput = $state('');

	onMount(() => {
		bookBrowseService.loadTrendingBooks('fiction');
	});

	async function handleSearch() {
		if (!searchInput.trim()) return;
		activeTab = 'search';
		await bookBrowseService.searchBooks(searchInput.trim());
	}

	function handleSelectBook(book: DisplayBook) {
		goto(`/books/${book.key}`);
	}

	function handleSubjectChange(subject: BookSubject) {
		bookBrowseService.loadTrendingBooks(subject);
	}

	function handleSearchPage(page: number) {
		if ($browseState.searchQuery) {
			bookBrowseService.searchBooks($browseState.searchQuery, page);
		}
	}

	function handleTrendingPage(page: number) {
		bookBrowseService.loadTrendingBooks($browseState.selectedSubject, page);
	}
</script>

<div class="flex h-full flex-col overflow-y-auto">
	<div class="flex items-center justify-between gap-4 border-b border-base-300 px-4 py-3">
		<h1 class="text-lg font-bold">Books</h1>
		<div class="flex items-center gap-2">
			<form class="join" onsubmit={(e) => { e.preventDefault(); handleSearch(); }}>
				<input
					type="text"
					placeholder="Search books..."
					class="input join-item input-bordered input-sm w-48"
					bind:value={searchInput}
				/>
				<button type="submit" class="btn join-item btn-sm btn-primary">Search</button>
			</form>
		</div>
	</div>

	<div role="tablist" class="tabs tabs-bordered px-4">
		<button
			role="tab"
			class={classNames('tab', { 'tab-active': activeTab === 'search' })}
			onclick={() => (activeTab = 'search')}
		>
			Search
		</button>
		<button
			role="tab"
			class={classNames('tab', { 'tab-active': activeTab === 'trending' })}
			onclick={() => (activeTab = 'trending')}
		>
			Trending
		</button>
	</div>

	{#if activeTab === 'trending'}
		<div class="flex flex-wrap gap-1 border-b border-base-300 px-4 py-2">
			{#each BOOK_SUBJECTS as subject}
				<button
					class={classNames('btn btn-xs', {
						'btn-primary': $browseState.selectedSubject === subject,
						'btn-ghost': $browseState.selectedSubject !== subject
					})}
					onclick={() => handleSubjectChange(subject)}
				>
					{BOOK_SUBJECT_LABELS[subject]}
				</button>
			{/each}
		</div>
		<BookBrowseGrid
			books={$browseState.trendingResults}
			selectedBookKey={null}
			page={$browseState.trendingPage}
			totalPages={$browseState.trendingTotalPages}
			loading={$browseState.loading}
			error={$browseState.error}
			onselect={handleSelectBook}
			onpage={handleTrendingPage}
		/>
	{:else}
		<BookBrowseGrid
			books={$browseState.searchResults}
			selectedBookKey={null}
			page={$browseState.searchPage}
			totalPages={$browseState.searchTotalPages}
			loading={$browseState.loading}
			error={$browseState.error}
			onselect={handleSelectBook}
			onpage={handleSearchPage}
		/>
	{/if}
</div>
