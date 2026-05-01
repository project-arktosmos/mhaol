<script lang="ts">
	import classNames from 'classnames';
	import type { MediaList } from 'ui-lib/types/media-list.type';
	import { libraryFileAdapter } from 'ui-lib/adapters/classes/library-file.adapter';
	import type { MediaType } from 'ui-lib/types/library.type';
	import type { DisplayTMDBTvShowDetails } from 'addons/tmdb/types';
	import type { DisplayMusicBrainzReleaseGroup } from 'addons/musicbrainz/types';

	interface Props {
		list: MediaList;
		selected?: boolean;
		tmdbMetadata?: DisplayTMDBTvShowDetails | null;
		mbMetadata?: DisplayMusicBrainzReleaseGroup | null;
		seasonCount?: number;
		href?: string;
		onselect?: (list: MediaList) => void;
	}

	let {
		list,
		selected = false,
		tmdbMetadata = null,
		mbMetadata = null,
		seasonCount,
		href,
		onselect
	}: Props = $props();

	let kindLabel = $derived(list.mediaType === 'video' ? 'TV Show' : 'Album');

	let coverUrl = $derived.by(() => {
		if (list.coverImage) return list.coverImage;
		const seasonNum = list.links.tmdb?.seasonNumber;
		if (tmdbMetadata && seasonNum != null) {
			const season = tmdbMetadata.seasons.find((s) => s.seasonNumber === seasonNum);
			if (season?.posterUrl) return season.posterUrl;
		}
		if (tmdbMetadata?.posterUrl) return tmdbMetadata.posterUrl;
		if (mbMetadata?.coverArtUrl) return mbMetadata.coverArtUrl;
		return null;
	});

	let subtitle = $derived.by(() => {
		if (tmdbMetadata) return tmdbMetadata.name;
		if (mbMetadata) return mbMetadata.artistCredits;
		return null;
	});
</script>

{#snippet cardBody()}
	<figure class="relative h-48 overflow-hidden bg-base-300">
		{#if coverUrl}
			<img src={coverUrl} alt={list.title} class="h-full w-full object-cover" loading="lazy" />
		{:else}
			<div class="flex h-full w-full items-center justify-center text-base-content/20">
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class="h-12 w-12"
					fill="none"
					viewBox="0 0 24 24"
					stroke="currentColor"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="1"
						d="M4 6h16M4 10h16M4 14h16M4 18h16"
					/>
				</svg>
			</div>
		{/if}
	</figure>
	<div class="card-body gap-1">
		<h3 class="card-title truncate text-sm" title={list.title}>{list.title}</h3>
		{#if subtitle}
			<p class="truncate text-xs opacity-60" title={subtitle}>{subtitle}</p>
		{/if}
		<div class="flex flex-wrap gap-1">
			<span
				class={classNames(
					'badge badge-xs',
					libraryFileAdapter.getMediaTypeBadgeClass(list.mediaType as MediaType)
				)}
			>
				{kindLabel}
			</span>
			<span class="badge badge-ghost badge-xs">{list.itemCount} items</span>
		</div>
	</div>
{/snippet}

{#if href}
	<a
		{href}
		class={classNames(
			'card-compact card cursor-pointer bg-base-200 text-inherit no-underline shadow-sm transition-shadow hover:shadow-md',
			{
				'ring-2 ring-primary': selected
			}
		)}
	>
		{@render cardBody()}
	</a>
{:else}
	<div
		class={classNames(
			'card-compact card cursor-pointer bg-base-200 shadow-sm transition-shadow hover:shadow-md',
			{
				'ring-2 ring-primary': selected
			}
		)}
		onclick={() => onselect?.(list)}
		role="button"
		tabindex={0}
		onkeydown={(e) => {
			if (e.key === 'Enter' || e.key === ' ') {
				e.preventDefault();
				onselect?.(list);
			}
		}}
	>
		{@render cardBody()}
	</div>
{/if}
