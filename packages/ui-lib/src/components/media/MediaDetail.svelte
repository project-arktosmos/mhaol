<script lang="ts">
	import classNames from 'classnames';
	import { libraryFileAdapter } from 'frontend/adapters/classes/library-file.adapter';
	import { getThumbnailUrl } from '$utils/youtube/embed';
	import TagPill from 'ui-lib/components/images/TagPill.svelte';
	import PlayerVideo from 'ui-lib/components/player/PlayerVideo.svelte';
	import { apiUrl } from 'frontend/lib/api-base';
	import { lyricsService } from 'frontend/services/lyrics.service';
	import { playerService } from 'frontend/services/player.service';
	import type { MediaDetailSelection } from 'frontend/types/media-detail.type';
	import type { MediaType } from 'frontend/types/library.type';
	import type { DisplayTMDBMovieDetails } from 'addons/tmdb/types';
	import type { DisplayTMDBTvShowDetails } from 'addons/tmdb/types';

	interface Props {
		selection: MediaDetailSelection;
		onclose?: () => void;
	}

	let { selection, onclose }: Props = $props();

	const lyricsState = lyricsService.store;
	let lastFetchedItemId: string | null = $state(null);

	$effect(() => {
		const isLinked = selection.cardType === 'audio' && !!selection.item.links.musicbrainz;
		if (isLinked && selection.item.id !== lastFetchedItemId) {
			lastFetchedItemId = selection.item.id;
			lyricsService.fetchForItemId(selection.item.id);
		} else if (!isLinked && lastFetchedItemId) {
			lastFetchedItemId = null;
			lyricsService.clear();
		}
	});

	let imageUrl = $derived.by(() => {
		const { cardType, item, tmdbMetadata, musicbrainzMetadata } = selection;
		if (cardType === 'movie' || cardType === 'tv') {
			return (
				(tmdbMetadata as DisplayTMDBMovieDetails | DisplayTMDBTvShowDetails | null)?.posterUrl ??
				null
			);
		}
		if (cardType === 'youtube') {
			const videoId = item.links.youtube?.serviceId ?? '';
			return videoId ? getThumbnailUrl(videoId) : null;
		}
		if (cardType === 'audio') {
			return musicbrainzMetadata?.coverArtUrl ?? null;
		}
		if (cardType === 'image') {
			return apiUrl(`/api/images/serve?path=${encodeURIComponent(item.path)}`);
		}
		return null;
	});

	let imageAlt = $derived.by(() => {
		const { cardType, tmdbMetadata, youtubeMetadata, musicbrainzMetadata, item } = selection;
		if (cardType === 'movie')
			return (tmdbMetadata as DisplayTMDBMovieDetails | null)?.title ?? item.name;
		if (cardType === 'tv')
			return (tmdbMetadata as DisplayTMDBTvShowDetails | null)?.name ?? item.name;
		if (cardType === 'youtube') return youtubeMetadata?.title ?? item.name;
		if (cardType === 'audio') return musicbrainzMetadata?.title ?? item.name;
		return item.name;
	});

	let isLinkedAudio = $derived(
		selection.cardType === 'audio' && !!selection.item.links.musicbrainz
	);

	const playerState = playerService.state;
	let isPlaying = $derived($playerState.currentFile?.id === selection.item.id);

	let newTagInput = $state('');

	function handleAddTag() {
		const tag = newTagInput.trim().toLowerCase();
		if (!tag) return;
		selection.onaddtag?.(selection.item, tag);
		newTagInput = '';
	}
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
				libraryFileAdapter.getMediaTypeBadgeClass(selection.item.mediaTypeId as MediaType)
			)}
		>
			{libraryFileAdapter.getMediaTypeLabel(selection.item.mediaTypeId as MediaType)}
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
	{:else if selection.cardType === 'youtube'}
		{#if selection.youtubeMetadata}
			<p class="text-xs font-semibold">{selection.youtubeMetadata.title}</p>
			<p class="text-xs opacity-60">{selection.youtubeMetadata.author_name}</p>
		{/if}
	{:else if selection.cardType === 'audio'}
		{#if selection.musicbrainzMetadata}
			<p class="text-xs font-medium">{selection.musicbrainzMetadata.title}</p>
			<p class="text-xs opacity-60">{selection.musicbrainzMetadata.artistCredits}</p>
			{#if selection.musicbrainzMetadata.firstReleaseTitle}
				<p class="text-xs opacity-50">{selection.musicbrainzMetadata.firstReleaseTitle}</p>
			{/if}
			{#if selection.musicbrainzMetadata.duration}
				<p class="text-xs opacity-40">{selection.musicbrainzMetadata.duration}</p>
			{/if}

			{@const lyrics = $lyricsState}
			{#if lyrics.status === 'loading'}
				<div class="flex items-center gap-2 py-2">
					<span class="loading loading-xs loading-spinner text-primary"></span>
					<span class="text-xs text-base-content/60">Fetching lyrics...</span>
				</div>
			{:else if lyrics.status === 'success' && lyrics.lyrics}
				<div class="flex flex-col gap-1 rounded-lg bg-base-300/50 px-3 py-2">
					<div class="flex items-center justify-between">
						<span class="text-xs font-semibold text-base-content/70">Lyrics</span>
						{#if lyrics.lyrics.syncedLyrics}
							<span class="badge badge-xs badge-primary">Synced</span>
						{/if}
					</div>
					{#if lyrics.lyrics.instrumental}
						<p class="py-2 text-center text-xs text-base-content/40">Instrumental</p>
					{:else if lyrics.lyrics.syncedLyrics && lyrics.lyrics.syncedLyrics.length > 0}
						<div class="max-h-48 space-y-0.5 overflow-y-auto">
							{#each lyrics.lyrics.syncedLyrics as line}
								<p class="text-xs text-base-content/60">
									{#if line.text}
										{line.text}
									{:else}
										<span class="text-base-content/20">...</span>
									{/if}
								</p>
							{/each}
						</div>
					{:else if lyrics.lyrics.plainLyrics}
						<div
							class="max-h-48 overflow-y-auto text-xs leading-relaxed whitespace-pre-wrap text-base-content/60"
						>
							{lyrics.lyrics.plainLyrics}
						</div>
					{/if}
				</div>
			{:else if lyrics.status === 'error'}
				<p class="text-xs text-error/60">{lyrics.error ?? 'Failed to load lyrics'}</p>
			{/if}
		{:else}
			<p class="text-xs opacity-60" title={selection.item.path}>{selection.item.path}</p>
		{/if}
	{:else if selection.cardType === 'image'}
		<p class="text-xs opacity-60" title={selection.item.path}>{selection.item.path}</p>
		{#if selection.imageTagging}
			<div class="flex items-center gap-2 text-xs opacity-70">
				<span class="loading loading-xs loading-spinner"></span>
				Tagging...
			</div>
		{/if}
		{#if selection.imageTags.length > 0}
			<div class="flex flex-wrap gap-1">
				{#each selection.imageTags as tag (tag.tag)}
					<TagPill
						tag={tag.tag}
						score={tag.score}
						onremove={(t) => selection.onremovetag?.(selection.item, t)}
					/>
				{/each}
			</div>
		{/if}
		<form
			class="flex gap-1"
			onsubmit={(e) => {
				e.preventDefault();
				handleAddTag();
			}}
		>
			<input
				type="text"
				placeholder="Add tag..."
				class="input-bordered input input-xs flex-1"
				bind:value={newTagInput}
			/>
			<button type="submit" class="btn btn-xs btn-primary" disabled={!newTagInput.trim()}
				>Add</button
			>
		</form>
	{:else}
		<p class="text-xs opacity-60" title={selection.item.path}>{selection.item.path}</p>
	{/if}

	<div class="flex flex-wrap gap-2">
		{#if selection.cardType === 'image'}
			<button
				class="btn btn-sm btn-primary"
				disabled={selection.imageTagging}
				onclick={() => selection.ontagimage?.(selection.item)}
			>
				{#if selection.imageTagging}
					<span class="loading loading-xs loading-spinner"></span>
				{/if}
				{selection.imageTags.length > 0 ? 'Re-tag' : 'Tag'}
			</button>
		{/if}
		{#if selection.cardType === 'movie' || selection.cardType === 'tv' || selection.cardType === 'youtube' || selection.cardType === 'video'}
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
				onclick={() => selection.onlink?.(selection.item, 'tmdb')}>Link metadata</button
			>
			<button
				class="btn btn-sm btn-info"
				onclick={() => selection.onlink?.(selection.item, 'youtube')}>Link YouTube</button
			>
		{/if}
		{#if selection.cardType === 'audio' && !isLinkedAudio}
			<button
				class="btn btn-sm btn-primary"
				onclick={() => selection.onlink?.(selection.item, 'musicbrainz')}>Link metadata</button
			>
		{/if}
		{#if selection.cardType === 'movie' || selection.cardType === 'tv'}
			<button
				class="btn btn-ghost btn-sm"
				onclick={() => selection.onunlink?.(selection.item, 'tmdb')}>Unlink</button
			>
		{/if}
		{#if selection.cardType === 'youtube'}
			<button
				class="btn btn-ghost btn-sm"
				onclick={() => selection.onunlink?.(selection.item, 'youtube')}>Unlink</button
			>
		{/if}
		{#if selection.cardType === 'audio' && isLinkedAudio}
			<button
				class="btn btn-ghost btn-sm"
				onclick={() => selection.onunlink?.(selection.item, 'musicbrainz')}>Unlink</button
			>
		{/if}
	</div>
</div>
