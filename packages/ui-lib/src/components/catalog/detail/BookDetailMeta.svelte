<script lang="ts">
	import { marked } from 'marked';
	import DOMPurify from 'dompurify';
	import type { CatalogBook } from 'ui-lib/types/catalog.type';

	interface Props {
		item: CatalogBook;
	}

	let { item }: Props = $props();

	let description = $derived(item.metadata.description);
	let renderedDescription = $derived(
		description ? DOMPurify.sanitize(marked.parse(description, { async: false }) as string) : null
	);
	let authors = $derived(item.metadata.authors);
	let subjects = $derived(item.metadata.subjects);
	let publishers = $derived(item.metadata.publishers);
	let pageCount = $derived(item.metadata.pageCount);
	let isbn = $derived(item.metadata.isbn);
	let editionCount = $derived(item.metadata.editionCount);
</script>

<div class="flex flex-col gap-3">
	{#if renderedDescription}
		<div>
			<h3 class="mb-1 text-xs font-semibold tracking-wide uppercase opacity-50">Description</h3>
			<div class="prose-sm prose max-w-none">
				{@html renderedDescription}
			</div>
		</div>
	{/if}

	{#if authors.length > 0}
		<div>
			<h3 class="mb-1 text-xs font-semibold tracking-wide uppercase opacity-50">Authors</h3>
			<div class="flex flex-wrap gap-1">
				{#each authors as author}
					<span class="badge badge-ghost">{author}</span>
				{/each}
			</div>
		</div>
	{/if}

	{#if subjects.length > 0}
		<div>
			<h3 class="mb-1 text-xs font-semibold tracking-wide uppercase opacity-50">Subjects</h3>
			<div class="flex flex-wrap gap-1">
				{#each subjects.slice(0, 10) as subject}
					<span class="badge badge-outline badge-xs">{subject}</span>
				{/each}
			</div>
		</div>
	{/if}

	<div class="grid grid-cols-2 gap-2 text-sm">
		{#if pageCount}
			<div>
				<span class="opacity-50">Pages:</span>
				<span class="font-medium">{pageCount}</span>
			</div>
		{/if}
		{#if isbn}
			<div>
				<span class="opacity-50">ISBN:</span>
				<span class="font-medium">{isbn}</span>
			</div>
		{/if}
		{#if editionCount > 0}
			<div>
				<span class="opacity-50">Editions:</span>
				<span class="font-medium">{editionCount}</span>
			</div>
		{/if}
		{#if publishers.length > 0}
			<div>
				<span class="opacity-50">Publisher:</span>
				<span class="font-medium">{publishers[0]}</span>
			</div>
		{/if}
	</div>
</div>
