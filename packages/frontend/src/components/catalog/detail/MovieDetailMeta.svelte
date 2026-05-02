<script lang="ts">
	import type { CatalogMovie } from '$types/catalog.type';
	import { authorsByRole } from '$types/catalog.type';
	import AuthorList from './AuthorList.svelte';

	interface Props {
		item: CatalogMovie;
		onshowimages?: () => void;
	}

	let { item, onshowimages }: Props = $props();

	let genres = $derived(item.metadata.genres);
	let authors = $derived(item.metadata.authors);
	let directors = $derived(authorsByRole(authors, 'director'));
	let cast = $derived(authorsByRole(authors, 'actor'));
	let runtime = $derived(item.metadata.runtime);
	let tagline = $derived(item.metadata.tagline);
	let budget = $derived(item.metadata.budget);
	let revenue = $derived(item.metadata.revenue);
	let images = $derived(item.metadata.images);
</script>

<div class="flex flex-col gap-3">
	{#if tagline}
		<p class="text-sm italic opacity-60">"{tagline}"</p>
	{/if}

	{#if genres.length > 0}
		<div class="flex flex-wrap gap-1">
			{#each genres as genre}
				<span class="badge badge-ghost badge-sm">{genre}</span>
			{/each}
		</div>
	{/if}

	<div class="grid grid-cols-2 gap-2 text-sm">
		{#if runtime}
			<div>
				<span class="opacity-50">Runtime:</span>
				<span class="font-medium">{runtime}</span>
			</div>
		{/if}
		{#if directors.length > 0}
			<AuthorList authors={directors} layout="labeled" label="Director" />
		{/if}
		{#if budget}
			<div>
				<span class="opacity-50">Budget:</span>
				<span class="font-medium">{budget}</span>
			</div>
		{/if}
		{#if revenue}
			<div>
				<span class="opacity-50">Revenue:</span>
				<span class="font-medium">{revenue}</span>
			</div>
		{/if}
	</div>

	{#if cast.length > 0}
		<AuthorList authors={cast} layout="grid" label="Cast" maxItems={10} />
	{/if}

	{#if images.length > 0 && onshowimages}
		<button class="btn w-full btn-outline btn-sm" onclick={onshowimages}>
			Show Images ({images.length})
		</button>
	{/if}
</div>
