<script lang="ts">
	import type { CatalogTvShow } from 'ui-lib/types/catalog.type';
	import { authorsByRole } from 'ui-lib/types/catalog.type';
	import type { SmartSearchTorrentResult } from 'ui-lib/types/smart-search.type';
	import AuthorList from './AuthorList.svelte';

	interface Props {
		item: CatalogTvShow;
		onseasonselect?: (seasonNumber: number) => void;
		completeCandidate?: SmartSearchTorrentResult | null;
		seasonCandidates?: Record<number, SmartSearchTorrentResult | null>;
	}

	let { item, onseasonselect, completeCandidate = null, seasonCandidates = {} }: Props = $props();

	let genres = $derived(item.metadata.genres);
	let authors = $derived(item.metadata.authors);
	let creators = $derived(authorsByRole(authors, 'creator'));
	let cast = $derived(authorsByRole(authors, 'actor'));
	let status = $derived(item.metadata.status);
	let networks = $derived(item.metadata.networks);
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
		{#if creators.length > 0}
			<AuthorList authors={creators} layout="labeled" label="Created by" />
		{/if}
	</div>

	{#if seasons.length > 0}
		<div>
			<h3 class="mb-1 text-xs font-semibold tracking-wide uppercase opacity-50">Seasons</h3>
			<div class="flex flex-col gap-1">
				{#each seasons as season}
					{@const seasonMatch = seasonCandidates?.[season.seasonNumber] ?? null}
					{@const coveredByComplete = completeCandidate !== null && season.seasonNumber > 0}
					<button
						class="flex flex-col gap-1 rounded-lg p-2 text-left text-sm hover:bg-base-200"
						onclick={() => onseasonselect?.(season.seasonNumber)}
					>
						<div class="flex items-center gap-2">
							{#if season.posterUrl}
								<img
									src={season.posterUrl}
									alt={season.name}
									class="h-12 w-8 rounded object-cover"
									loading="lazy"
								/>
							{/if}
							<div class="flex-1">
								<div class="flex items-center gap-2">
									<p class="font-medium">{season.name}</p>
									{#if seasonMatch}
										<span class="badge badge-xs badge-success">match</span>
									{:else if coveredByComplete}
										<span class="badge badge-xs badge-info">in complete</span>
									{/if}
								</div>
								<p class="text-xs opacity-50">
									{season.episodeCount} episodes{season.airDate ? ` · ${season.airDate}` : ''}
								</p>
							</div>
						</div>
						{#if seasonMatch}
							<p class="truncate pl-10 text-xs opacity-60">
								{seasonMatch.name}
								{#if seasonMatch.analysis?.quality}
									<span class="ml-1 opacity-50">· {seasonMatch.analysis.quality}</span>
								{/if}
							</p>
						{/if}
					</button>
				{/each}
			</div>
		</div>
	{/if}

	{#if cast.length > 0}
		<AuthorList authors={cast} layout="grid" label="Cast" maxItems={10} />
	{/if}
</div>
