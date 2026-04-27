<script lang="ts">
	import type { CatalogTvShow } from 'ui-lib/types/catalog.type';
	import { authorsByRole } from 'ui-lib/types/catalog.type';
	import type { SmartSearchTorrentResult, TvEpisodeMeta } from 'ui-lib/types/smart-search.type';
	import type { TorrentInfo } from 'ui-lib/types/torrent.type';
	import {
		formatBytes,
		formatSpeed,
		formatEta,
		getStateLabel,
		getStateColor
	} from 'ui-lib/types/torrent.type';
	import AuthorList from './AuthorList.svelte';

	interface Props {
		item: CatalogTvShow;
		completeCandidate?: SmartSearchTorrentResult | null;
		seasonCandidates?: Record<number, SmartSearchTorrentResult | null>;
		seasonEpisodes?: Record<number, TvEpisodeMeta[]>;
		torrentByHash?: Record<string, TorrentInfo>;
	}

	let {
		item,
		completeCandidate = null,
		seasonCandidates = {},
		seasonEpisodes = {},
		torrentByHash = {}
	}: Props = $props();

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

{#snippet torrentProgress(torrent: TorrentInfo)}
	{@const percent = Math.round(torrent.progress * 100)}
	<div class="flex flex-col gap-1">
		<div class="flex items-center justify-between text-xs">
			<span class={getStateColor(torrent.state)}>{getStateLabel(torrent.state)}</span>
			<span class="font-mono">{percent}%</span>
		</div>
		<progress class="progress w-full progress-primary" value={torrent.progress} max="1"
		></progress>
		<div class="flex flex-wrap gap-3 text-xs opacity-60">
			<span>↓ {formatSpeed(torrent.downloadSpeed)}</span>
			<span>↑ {formatSpeed(torrent.uploadSpeed)}</span>
			<span>{formatBytes(torrent.size)}</span>
			{#if torrent.eta}<span>ETA {formatEta(torrent.eta)}</span>{/if}
		</div>
	</div>
{/snippet}

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

	{#if completeCandidate}
		{@const completeTorrent = torrentByHash[completeCandidate.infoHash.toLowerCase()]}
		<div class="rounded-lg border border-base-300 p-2">
			<div class="flex items-center gap-2">
				<span class="badge badge-xs badge-primary">Complete series</span>
				<p class="flex-1 truncate text-xs">{completeCandidate.name}</p>
			</div>
			{#if completeTorrent}
				<div class="mt-2">
					{@render torrentProgress(completeTorrent)}
				</div>
			{:else}
				<p class="mt-1 text-xs opacity-50">Queued…</p>
			{/if}
		</div>
	{/if}

	{#if seasons.length > 0}
		<div>
			<h3 class="mb-1 text-xs font-semibold tracking-wide uppercase opacity-50">Seasons</h3>
			<div class="flex flex-col gap-2">
				{#each seasons as season}
					{@const seasonMatch = seasonCandidates?.[season.seasonNumber] ?? null}
					{@const coveredByComplete = completeCandidate !== null && season.seasonNumber > 0}
					{@const seasonTorrent = seasonMatch
						? torrentByHash[seasonMatch.infoHash.toLowerCase()]
						: undefined}
					{@const episodes = seasonEpisodes?.[season.seasonNumber] ?? []}
					<div class="rounded-lg border border-base-300 p-2">
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
							<p class="mt-2 truncate text-xs opacity-60">{seasonMatch.name}</p>
							{#if seasonTorrent}
								<div class="mt-2">
									{@render torrentProgress(seasonTorrent)}
								</div>
							{:else}
								<p class="mt-1 text-xs opacity-50">Queued…</p>
							{/if}
						{/if}
						{#if episodes.length > 0}
							<div class="mt-2 flex flex-col">
								{#each episodes as ep}
									<div
										class="flex items-center justify-between border-b border-base-200 py-1 text-xs last:border-0"
									>
										<span class="flex items-center gap-2">
											<span class="w-10 font-mono opacity-50"
												>S{String(ep.seasonNumber).padStart(2, '0')}E{String(
													ep.episodeNumber
												).padStart(2, '0')}</span
											>
											<span class="truncate">{ep.name}</span>
										</span>
									</div>
								{/each}
							</div>
						{/if}
					</div>
				{/each}
			</div>
		</div>
	{/if}

	{#if cast.length > 0}
		<AuthorList authors={cast} layout="grid" label="Cast" maxItems={10} />
	{/if}
</div>
