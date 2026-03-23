<script lang="ts">
	import type { DisplayBook } from 'addons/openlibrary/types';
	import BookCard from './BookCard.svelte';

	interface Props {
		books: DisplayBook[];
		selectedBookKey?: string | null;
		page?: number;
		totalPages?: number;
		loading?: boolean;
		error?: string | null;
		onselect?: (book: DisplayBook) => void;
		onpage?: (page: number) => void;
	}

	let {
		books,
		selectedBookKey = null,
		page = 1,
		totalPages = 1,
		loading = false,
		error = null,
		onselect,
		onpage
	}: Props = $props();
</script>

{#if loading}
	<div class="flex items-center justify-center py-16">
		<span class="loading loading-lg loading-spinner"></span>
	</div>
{:else if error}
	<div class="flex items-center justify-center py-16">
		<p class="text-error">{error}</p>
	</div>
{:else if books.length === 0}
	<div class="flex items-center justify-center py-16">
		<p class="text-base-content/50">No books found</p>
	</div>
{:else}
	<div
		class="grid grid-cols-2 gap-4 p-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6"
	>
		{#each books as book (book.key)}
			<BookCard {book} selected={selectedBookKey === book.key} {onselect} />
		{/each}
	</div>

	{#if totalPages > 1}
		<div class="flex items-center justify-center gap-2 p-4">
			<button class="btn btn-sm" disabled={page <= 1} onclick={() => onpage?.(page - 1)}>
				Prev
			</button>
			<span class="text-sm opacity-60">Page {page} of {totalPages}</span>
			<button class="btn btn-sm" disabled={page >= totalPages} onclick={() => onpage?.(page + 1)}>
				Next
			</button>
		</div>
	{/if}
{/if}
