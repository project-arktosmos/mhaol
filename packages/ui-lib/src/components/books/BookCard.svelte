<script lang="ts">
	import classNames from 'classnames';
	import type { DisplayBook } from 'addons/openlibrary/types';
	import { apiUrl } from 'ui-lib/lib/api-base';

	interface Props {
		book: DisplayBook;
		selected?: boolean;
		onselect?: (book: DisplayBook) => void;
	}

	let { book, selected = false, onselect }: Props = $props();

	let imgError = $state(false);

	let coverSrc = $derived(book.coverId ? apiUrl(`/api/openlibrary/cover/${book.coverId}/M`) : null);
</script>

<div
	class={classNames('card-compact card bg-base-200 shadow-sm', {
		'ring-2 ring-primary': selected,
		'cursor-pointer transition-shadow hover:shadow-md': !!onselect
	})}
	onclick={() => onselect?.(book)}
	role={onselect ? 'button' : undefined}
	tabindex={onselect ? 0 : undefined}
	onkeydown={onselect
		? (e) => {
				if (e.key === 'Enter' || e.key === ' ') {
					e.preventDefault();
					onselect?.(book);
				}
			}
		: undefined}
>
	<figure class="relative aspect-[2/3] overflow-hidden bg-base-300">
		{#if coverSrc && !imgError}
			<img
				src={coverSrc}
				alt={book.title}
				class="h-full w-full object-cover"
				loading="lazy"
				onerror={() => (imgError = true)}
			/>
		{:else}
			<div class="flex h-full w-full items-center justify-center text-base-content/20">
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class="h-12 w-12"
					fill="none"
					viewBox="0 0 24 24"
					stroke="currentColor"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="1.5"
						d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253"
					/>
				</svg>
			</div>
		{/if}
	</figure>
	<div class="card-body gap-0.5">
		<h3 class="card-title truncate text-sm" title={book.title}>{book.title}</h3>
		<p class="truncate text-xs opacity-60" title={book.authors.join(', ')}>
			{book.authors.join(', ') || 'Unknown Author'}
		</p>
		<div class="flex items-center gap-1">
			{#if book.firstPublishYear}
				<span class="text-xs opacity-40">{book.firstPublishYear}</span>
			{/if}
			{#if book.ratingsAverage}
				<span class="badge badge-ghost badge-xs">{book.ratingsAverage.toFixed(1)}</span>
			{/if}
		</div>
	</div>
</div>
