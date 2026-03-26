<script lang="ts">
	import classNames from 'classnames';
	import type { CatalogMovie } from 'ui-lib/types/catalog.type';

	interface Props {
		item: CatalogMovie;
		onshowimages?: () => void;
	}

	let { item, onshowimages }: Props = $props();

	let genres = $derived(item.metadata.genres);
	let cast = $derived(item.metadata.cast);
	let director = $derived(item.metadata.director);
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
		{#if director}
			<div>
				<span class="opacity-50">Director:</span>
				<span class="font-medium">{director}</span>
			</div>
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
		<div>
			<h3 class="mb-1 text-xs font-semibold tracking-wide uppercase opacity-50">Cast</h3>
			<div class="grid grid-cols-2 gap-1 text-sm">
				{#each cast.slice(0, 10) as member}
					<div class="flex items-center gap-2">
						{#if member.profileUrl}
							<img
								src={member.profileUrl}
								alt={member.name}
								class="h-8 w-8 rounded-full object-cover"
								loading="lazy"
							/>
						{:else}
							<div
								class="flex h-8 w-8 items-center justify-center rounded-full bg-base-300 text-xs"
							>
								{member.name[0]}
							</div>
						{/if}
						<div>
							<p class="text-xs font-medium">{member.name}</p>
							<p class="text-xs opacity-50">{member.character}</p>
						</div>
					</div>
				{/each}
			</div>
		</div>
	{/if}

	{#if images.length > 0 && onshowimages}
		<button class="btn w-full btn-outline btn-sm" onclick={onshowimages}>
			Show Images ({images.length})
		</button>
	{/if}
</div>
