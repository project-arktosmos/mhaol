<script lang="ts">
	import type { CatalogTvShow } from 'ui-lib/types/catalog.type';

	interface Props {
		item: CatalogTvShow;
		onseasonselect?: (seasonNumber: number) => void;
	}

	let { item, onseasonselect }: Props = $props();

	let genres = $derived(item.metadata.genres);
	let cast = $derived(item.metadata.cast);
	let status = $derived(item.metadata.status);
	let networks = $derived(item.metadata.networks);
	let createdBy = $derived(item.metadata.createdBy);
	let seasons = $derived(item.metadata.seasons);
	let tagline = $derived(item.metadata.tagline);
	let numberOfSeasons = $derived(item.metadata.numberOfSeasons);
	let numberOfEpisodes = $derived(item.metadata.numberOfEpisodes);
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
		{#if status}
			<div>
				<span class="opacity-50">Status:</span>
				<span class="font-medium">{status}</span>
			</div>
		{/if}
		{#if numberOfSeasons}
			<div>
				<span class="opacity-50">Seasons:</span>
				<span class="font-medium">{numberOfSeasons}</span>
			</div>
		{/if}
		{#if numberOfEpisodes}
			<div>
				<span class="opacity-50">Episodes:</span>
				<span class="font-medium">{numberOfEpisodes}</span>
			</div>
		{/if}
		{#if networks.length > 0}
			<div>
				<span class="opacity-50">Network:</span>
				<span class="font-medium">{networks.join(', ')}</span>
			</div>
		{/if}
		{#if createdBy.length > 0}
			<div>
				<span class="opacity-50">Created by:</span>
				<span class="font-medium">{createdBy.join(', ')}</span>
			</div>
		{/if}
	</div>

	{#if seasons.length > 0}
		<div>
			<h3 class="mb-1 text-xs font-semibold tracking-wide uppercase opacity-50">Seasons</h3>
			<div class="flex flex-col gap-1">
				{#each seasons as season}
					<button
						class="flex items-center gap-2 rounded-lg p-2 text-left text-sm hover:bg-base-200"
						onclick={() => onseasonselect?.(season.seasonNumber)}
					>
						{#if season.posterUrl}
							<img src={season.posterUrl} alt={season.name} class="h-12 w-8 rounded object-cover" loading="lazy" />
						{/if}
						<div>
							<p class="font-medium">{season.name}</p>
							<p class="text-xs opacity-50">{season.episodeCount} episodes{season.airDate ? ` · ${season.airDate}` : ''}</p>
						</div>
					</button>
				{/each}
			</div>
		</div>
	{/if}

	{#if cast.length > 0}
		<div>
			<h3 class="mb-1 text-xs font-semibold tracking-wide uppercase opacity-50">Cast</h3>
			<div class="grid grid-cols-2 gap-1 text-sm">
				{#each cast.slice(0, 10) as member}
					<div class="flex items-center gap-2">
						{#if member.profileUrl}
							<img src={member.profileUrl} alt={member.name} class="h-8 w-8 rounded-full object-cover" loading="lazy" />
						{:else}
							<div class="flex h-8 w-8 items-center justify-center rounded-full bg-base-300 text-xs">{member.name[0]}</div>
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
</div>
