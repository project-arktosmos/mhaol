<script lang="ts">
	import classNames from 'classnames';
	import { marked } from 'marked';
	import DOMPurify from 'dompurify';
	import DetailPageLayout from 'ui-lib/components/core/DetailPageLayout.svelte';
	import { apiUrl } from 'ui-lib/lib/api-base';
	import type { DisplayBook, DisplayBookDetails } from 'addons/openlibrary/types';

	function renderMarkdown(text: string): string {
		return DOMPurify.sanitize(marked.parse(text, { async: false }) as string);
	}

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
		downloadStatus: { state: string; progress: number } | null;
		fetchedTorrent: { name: string; quality: string; languages: string } | null;
		onfetch: () => void;
		ondownload: () => void;
		onshowsearch: () => void;
		onback: () => void;
	}

	let {
		book,
		bookDetails,
		loading,
		fetching,
		fetched,
		fetchSteps,
		downloadStatus,
		fetchedTorrent,
		onfetch,
		ondownload,
		onshowsearch,
		onback
	}: Props = $props();

	let dlState = $derived(downloadStatus?.state ?? null);
	let isDownloading = $derived(
		dlState === 'downloading' ||
			dlState === 'initializing' ||
			dlState === 'paused' ||
			dlState === 'checking'
	);
	let isDownloaded = $derived(dlState === 'completed' || dlState === 'seeding');
	let downloadButtonDisabled = $derived(!fetched || isDownloading || isDownloaded);
	let dlProgress = $derived(downloadStatus?.progress ?? 0);
	let dlPercent = $derived(Math.round(dlProgress * 100));
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

	<div class="grid grid-cols-2 gap-2">
		<button
			class="btn col-span-2 btn-sm {fetched ? 'btn-ghost' : 'btn-info'}"
			onclick={onfetch}
			disabled={fetching}
		>
			{#if fetching}
				<span class="loading loading-xs loading-spinner"></span>
			{:else}
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class="h-4 w-4"
					fill="none"
					viewBox="0 0 24 24"
					stroke="currentColor"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
					/>
				</svg>
			{/if}
			Smart Search
		</button>
		{#if fetchSteps}
			<button
				class="col-span-2 cursor-pointer rounded-lg bg-base-200 p-2 transition-colors hover:bg-base-300"
				onclick={onshowsearch}
			>
				<ul class="steps steps-horizontal w-full text-xs">
					<li class={classNames('step', { 'step-success': fetchSteps.terms })}>Terms</li>
					<li class={classNames('step', { 'step-success': fetchSteps.search })}>
						{fetchSteps.searching ? 'Searching...' : 'Search'}
					</li>
					<li class={classNames('step', { 'step-success': fetchSteps.eval })}>Analysis</li>
					<li class={classNames('step', { 'step-success': fetchSteps.done })}>
						{fetchSteps.done ? 'Done' : 'Candidate'}
					</li>
				</ul>
			</button>
		{/if}
		{#if fetchedTorrent}
			<div class="col-span-2 flex items-center gap-2">
				<p class="min-w-0 flex-1 truncate text-xs opacity-60" title={fetchedTorrent.name}>
					{fetchedTorrent.name}
				</p>
				{#if fetchedTorrent.quality}
					<span class="badge badge-xs badge-info">{fetchedTorrent.quality}</span>
				{/if}
				{#if fetchedTorrent.languages}
					<span class="badge badge-ghost badge-xs">{fetchedTorrent.languages}</span>
				{/if}
			</div>
		{/if}
		<button
			class={classNames('btn col-span-2 btn-sm', {
				'btn-ghost': isDownloaded,
				'btn-success': !isDownloaded
			})}
			onclick={ondownload}
			disabled={downloadButtonDisabled}
		>
			{#if isDownloading}
				<span class="loading loading-xs loading-spinner"></span> Downloading
			{:else if isDownloaded}
				Downloaded
			{:else}
				Download
			{/if}
		</button>
		{#if isDownloading || isDownloaded}
			<div class="col-span-2 flex items-center gap-2">
				<progress
					class={classNames('progress flex-1', {
						'progress-info': isDownloading,
						'progress-success': isDownloaded
					})}
					value={dlPercent}
					max="100"
				></progress>
				<span class="text-xs font-medium opacity-60">{dlPercent}%</span>
			</div>
		{/if}
	</div>

	{#if loading}
		<div class="flex items-center justify-center py-4">
			<span class="loading loading-sm loading-spinner"></span>
		</div>
	{/if}

	{#if bookDetails?.description}
		<div class="divider my-0 text-xs opacity-50">Description</div>
		<div
			class="max-w-none text-sm leading-relaxed opacity-80 [&_a]:text-primary [&_a]:underline [&_blockquote]:border-l-2 [&_blockquote]:border-base-300 [&_blockquote]:pl-4 [&_blockquote]:italic [&_h1]:text-lg [&_h1]:font-bold [&_h2]:text-base [&_h2]:font-semibold [&_h3]:text-sm [&_h3]:font-semibold [&_ol]:list-decimal [&_ol]:pl-5 [&_p+p]:mt-2 [&_ul]:list-disc [&_ul]:pl-5"
		>
			{@html renderMarkdown(bookDetails.description)}
		</div>
	{/if}

	{#if book.subjects.length > 0}
		<div class="divider my-0 text-xs opacity-50">Subjects</div>
		<div class="flex flex-wrap gap-1">
			{#each book.subjects.slice(0, 8) as subject}
				<span class="badge badge-ghost badge-sm">{subject}</span>
			{/each}
		</div>
	{/if}
</DetailPageLayout>
