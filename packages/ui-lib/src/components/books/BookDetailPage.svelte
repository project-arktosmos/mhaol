<script lang="ts">
	import classNames from 'classnames';
	import DetailPageLayout from 'ui-lib/components/core/DetailPageLayout.svelte';
	import { apiUrl } from 'ui-lib/lib/api-base';
	import type { DisplayBook, DisplayBookDetails } from 'addons/openlibrary/types';

	interface Props {
		book: DisplayBook;
		bookDetails: DisplayBookDetails | null;
		loading: boolean;
		fetching: boolean;
		fetched: boolean;
		fetchSteps: {
			terms: boolean;
			search: boolean;
			searching: boolean;
			eval: boolean;
			done: boolean;
		} | null;
		onfetch: () => void;
		ondownload: () => void;
		onback: () => void;
	}

	let {
		book,
		bookDetails,
		loading,
		fetching,
		fetched,
		fetchSteps,
		onfetch,
		ondownload,
		onback
	}: Props = $props();
</script>

<DetailPageLayout>
	<button class="btn self-start btn-ghost btn-sm" onclick={onback}>
		<svg
			xmlns="http://www.w3.org/2000/svg"
			class="h-4 w-4"
			fill="none"
			viewBox="0 0 24 24"
			stroke="currentColor"
			stroke-width="2"
		>
			<path stroke-linecap="round" stroke-linejoin="round" d="M15 19l-7-7 7-7" />
		</svg>
		Back
	</button>

	{#if book.coverUrl}
		<figure class="overflow-hidden rounded-lg bg-base-300">
			<img
				src={apiUrl(`/api/openlibrary/cover/${book.coverId}/L`)}
				alt={book.title}
				class="h-auto w-full max-w-sm object-contain"
			/>
		</figure>
	{/if}

	<div class="flex flex-col gap-1">
		<h1 class="text-xl font-bold">{book.title}</h1>
		<p class="text-sm opacity-70">
			{book.authors.join(', ') || 'Unknown Author'}
		</p>
		{#if book.firstPublishYear}
			<p class="text-sm opacity-50">First published: {book.firstPublishYear}</p>
		{/if}
		{#if book.pageCount}
			<p class="text-sm opacity-50">{book.pageCount} pages</p>
		{/if}
		{#if book.isbn}
			<p class="text-sm opacity-40">ISBN: {book.isbn}</p>
		{/if}
		{#if book.ratingsAverage}
			<p class="text-sm opacity-50">
				Rating: {book.ratingsAverage.toFixed(1)} ({book.ratingsCount} ratings)
			</p>
		{/if}
	</div>

	{#if loading}
		<div class="flex items-center justify-center py-4">
			<span class="loading loading-sm loading-spinner"></span>
		</div>
	{/if}

	{#if bookDetails?.description}
		<div class="divider my-0 text-xs opacity-50">Description</div>
		<p class="text-sm leading-relaxed opacity-80">
			{bookDetails.description}
		</p>
	{/if}

	{#if book.subjects.length > 0}
		<div class="divider my-0 text-xs opacity-50">Subjects</div>
		<div class="flex flex-wrap gap-1">
			{#each book.subjects.slice(0, 8) as subject}
				<span class="badge badge-ghost badge-sm">{subject}</span>
			{/each}
		</div>
	{/if}

	<div class="divider my-0 text-xs opacity-50">Actions</div>
	<div class="flex flex-col gap-2">
		{#if fetchSteps}
			<div class="flex items-center gap-2 text-xs">
				<span class={classNames(fetchSteps.search ? 'text-success' : 'opacity-50')}>Search</span>
				<span class="opacity-30">&rarr;</span>
				<span
					class={classNames(
						fetchSteps.eval ? 'text-success' : fetchSteps.searching ? 'text-info' : 'opacity-50'
					)}>Eval</span
				>
				<span class="opacity-30">&rarr;</span>
				<span class={classNames(fetchSteps.done ? 'text-success' : 'opacity-50')}>Done</span>
			</div>
		{/if}
		<button class="btn btn-sm btn-primary" onclick={onfetch} disabled={fetching}>
			{#if fetching}
				<span class="loading loading-xs loading-spinner"></span>
			{/if}
			{fetched ? 'Re-fetch' : 'Fetch'}
		</button>
		{#if fetched}
			<button class="btn btn-sm btn-success" onclick={ondownload}>Download</button>
		{/if}
	</div>
</DetailPageLayout>
