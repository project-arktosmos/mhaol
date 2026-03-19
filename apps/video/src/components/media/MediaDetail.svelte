<script lang="ts">
	import classNames from 'classnames';
	import { libraryFileAdapter } from '$adapters/classes/library-file.adapter';
	import PlayerVideo from '$components/player/PlayerVideo.svelte';
	import { playerService } from '$services/player.service';
	import type { MediaDetailSelection } from '$types/media-detail.type';
	import type { DisplayTMDBMovieDetails } from 'addons/tmdb/types';
	import type { DisplayTMDBTvShowDetails } from 'addons/tmdb/types';

	interface Props {
		selection: MediaDetailSelection;
		onclose?: () => void;
	}

	let { selection, onclose }: Props = $props();

	let imageUrl = $derived.by(() => {
		const { cardType, tmdbMetadata } = selection;
		if (cardType === 'movie' || cardType === 'tv') {
			return (
				(tmdbMetadata as DisplayTMDBMovieDetails | DisplayTMDBTvShowDetails | null)?.posterUrl ??
				null
			);
		}
		return null;
	});

	let imageAlt = $derived.by(() => {
		const { cardType, tmdbMetadata, item } = selection;
		if (cardType === 'movie')
			return (tmdbMetadata as DisplayTMDBMovieDetails | null)?.title ?? item.name;
		if (cardType === 'tv')
			return (tmdbMetadata as DisplayTMDBTvShowDetails | null)?.name ?? item.name;
		return item.name;
	});

	const playerState = playerService.state;
	let isPlaying = $derived($playerState.currentFile?.id === selection.item.id);
</script>

<div class="flex flex-col gap-3">
	<div class="flex items-center justify-between">
		<h2 class="text-sm font-semibold tracking-wide text-base-content/50 uppercase">Detail</h2>
		<button
			class="btn btn-square btn-ghost btn-xs"
			onclick={() => onclose?.()}
			aria-label="Close detail"
		>
			&times;
		</button>
	</div>

	<figure class="relative overflow-hidden rounded-lg bg-base-300">
		{#if isPlaying && $playerState.currentFile}
			<PlayerVideo
				file={$playerState.currentFile}
				connectionState={$playerState.connectionState}
				positionSecs={$playerState.positionSecs}
				durationSecs={$playerState.durationSecs}
			/>
		{:else if imageUrl}
			<img src={imageUrl} alt={imageAlt} class="w-full object-cover" />
		{:else}
			<div class="flex h-40 w-full items-center justify-center text-base-content/20">
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class="h-16 w-16"
					fill="none"
					viewBox="0 0 24 24"
					stroke="currentColor"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="1"
						d="M7 4v16M17 4v16M3 8h4m10 0h4M3 12h18M3 16h4m10 0h4M4 20h16a1 1 0 001-1V5a1 1 0 00-1-1H4a1 1 0 00-1 1v14a1 1 0 001 1z"
					/>
				</svg>
			</div>
		{/if}
	</figure>

	<h3 class="text-base font-semibold" title={selection.item.name}>{selection.item.name}</h3>

	<div class="flex flex-wrap gap-1">
		<span
			class={classNames(
				'badge badge-xs',
				libraryFileAdapter.getMediaTypeBadgeClass(selection.item.mediaTypeId)
			)}
		>
			{libraryFileAdapter.getMediaTypeLabel(selection.item.mediaTypeId)}
		</span>
		{#if selection.item.categoryId}
			<span
				class={classNames(
					'badge badge-xs',
					libraryFileAdapter.getCategoryBadgeClass(selection.item.categoryId)
				)}
			>
				{libraryFileAdapter.getCategoryLabel(selection.item.categoryId)}
			</span>
		{/if}
		<span class="badge badge-ghost badge-xs">{selection.item.extension}</span>
	</div>

	{#if selection.cardType === 'movie'}
		{@const metadata = selection.tmdbMetadata as DisplayTMDBMovieDetails | null}
		{#if metadata}
			<div class="flex flex-wrap items-center gap-1 text-xs">
				<span class="font-semibold">{metadata.title}</span>
				<span class="opacity-60">{metadata.releaseYear}</span>
				{#if metadata.voteAverage > 0}
					<span class="flex items-center gap-0.5">
						<svg
							xmlns="http://www.w3.org/2000/svg"
							viewBox="0 0 24 24"
							fill="currentColor"
							class="h-3 w-3 text-yellow-500"
						>
							<path
								fill-rule="evenodd"
								d="M10.788 3.21c.448-1.077 1.976-1.077 2.424 0l2.082 5.007 5.404.433c1.164.093 1.636 1.545.749 2.305l-4.117 3.527 1.257 5.273c.271 1.136-.964 2.033-1.96 1.425L12 18.354 7.373 21.18c-.996.608-2.231-.29-1.96-1.425l1.257-5.273-4.117-3.527c-.887-.76-.415-2.212.749-2.305l5.404-.433 2.082-5.006z"
								clip-rule="evenodd"
							/>
						</svg>
						<span class="font-semibold">{metadata.voteAverage.toFixed(1)}</span>
					</span>
				{/if}
			</div>
			{#if metadata.tagline}
				<p class="text-xs italic opacity-60">{metadata.tagline}</p>
			{/if}
			{#if metadata.genres.length > 0}
				<div class="flex flex-wrap gap-1">
					{#each metadata.genres as genre}
						<span class="badge badge-outline badge-xs badge-primary">{genre}</span>
					{/each}
				</div>
			{/if}
			{#if metadata.runtime}
				<p class="text-xs opacity-70">{metadata.runtime}</p>
			{/if}
			{#if metadata.director}
				<p class="text-xs opacity-70">Directed by {metadata.director}</p>
			{/if}
			{#if metadata.cast.length > 0}
				<div class="text-xs opacity-70">
					<span class="font-medium">Cast:</span>
					{metadata.cast
						.slice(0, 6)
						.map((c) => c.name)
						.join(', ')}
				</div>
			{/if}
			{#if metadata.overview}
				<p class="text-xs opacity-60">{metadata.overview}</p>
			{/if}
		{/if}
	{:else if selection.cardType === 'tv'}
		{@const metadata = selection.tmdbMetadata as DisplayTMDBTvShowDetails | null}
		{#if metadata}
			<div class="flex flex-wrap items-center gap-1 text-xs">
				<span class="font-semibold">{metadata.name}</span>
				<span class="opacity-60">
					{metadata.firstAirYear}{metadata.lastAirYear ? ` - ${metadata.lastAirYear}` : ''}
				</span>
				{#if metadata.voteAverage > 0}
					<span class="flex items-center gap-0.5">
						<svg
							xmlns="http://www.w3.org/2000/svg"
							viewBox="0 0 24 24"
							fill="currentColor"
							class="h-3 w-3 text-yellow-500"
						>
							<path
								fill-rule="evenodd"
								d="M10.788 3.21c.448-1.077 1.976-1.077 2.424 0l2.082 5.007 5.404.433c1.164.093 1.636 1.545.749 2.305l-4.117 3.527 1.257 5.273c.271 1.136-.964 2.033-1.96 1.425L12 18.354 7.373 21.18c-.996.608-2.231-.29-1.96-1.425l1.257-5.273-4.117-3.527c-.887-.76-.415-2.212.749-2.305l5.404-.433 2.082-5.006z"
								clip-rule="evenodd"
							/>
						</svg>
						<span class="font-semibold">{metadata.voteAverage.toFixed(1)}</span>
					</span>
				{/if}
			</div>
			{#if metadata.status}
				<span class="badge badge-outline badge-xs">{metadata.status}</span>
			{/if}
			{#if metadata.tagline}
				<p class="text-xs italic opacity-60">{metadata.tagline}</p>
			{/if}
			{#if metadata.genres.length > 0}
				<div class="flex flex-wrap gap-1">
					{#each metadata.genres as genre}
						<span class="badge badge-outline badge-xs badge-primary">{genre}</span>
					{/each}
				</div>
			{/if}
			{#if metadata.numberOfSeasons || metadata.numberOfEpisodes}
				<div class="flex gap-2 text-xs opacity-70">
					{#if metadata.numberOfSeasons}
						<span>{metadata.numberOfSeasons} season{metadata.numberOfSeasons !== 1 ? 's' : ''}</span
						>
					{/if}
					{#if metadata.numberOfEpisodes}
						<span>{metadata.numberOfEpisodes} ep{metadata.numberOfEpisodes !== 1 ? 's' : ''}</span>
					{/if}
				</div>
			{/if}
			{#if metadata.createdBy.length > 0}
				<p class="text-xs opacity-70">Created by {metadata.createdBy.join(', ')}</p>
			{/if}
			{#if metadata.cast.length > 0}
				<div class="text-xs opacity-70">
					<span class="font-medium">Cast:</span>
					{metadata.cast
						.slice(0, 6)
						.map((c) => c.name)
						.join(', ')}
				</div>
			{/if}
			{#if metadata.overview}
				<p class="text-xs opacity-60">{metadata.overview}</p>
			{/if}
		{/if}
	{:else}
		<p class="text-xs opacity-60" title={selection.item.path}>{selection.item.path}</p>
	{/if}

	<div class="flex flex-wrap gap-2">
		{#if selection.cardType === 'movie' || selection.cardType === 'tv' || selection.cardType === 'video'}
			{#if isPlaying}
				<button class="btn btn-ghost btn-sm" onclick={() => playerService.stop()}>Stop</button>
			{:else}
				<button class="btn btn-sm btn-accent" onclick={() => selection.onplay?.(selection.item)}
					>Play</button
				>
			{/if}
		{/if}
		{#if selection.cardType === 'video'}
			<button
				class="btn btn-sm btn-primary"
				onclick={() => selection.onlink?.(selection.item, 'tmdb-movie')}>Link Movie</button
			>
			<button
				class="btn btn-sm btn-primary"
				onclick={() => selection.onlink?.(selection.item, 'tmdb-tv')}>Link TV Show</button
			>
		{/if}
		{#if selection.cardType === 'movie' || selection.cardType === 'tv'}
			<button
				class="btn btn-ghost btn-sm"
				onclick={() => selection.onunlink?.(selection.item, 'tmdb')}>Unlink</button
			>
		{/if}
	</div>
</div>
