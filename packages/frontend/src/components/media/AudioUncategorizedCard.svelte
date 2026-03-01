<script lang="ts">
	import MediaCardBase from './MediaCardBase.svelte';
	import type { MediaItem } from '$types/media-card.type';
	import type { DisplayMusicBrainzRecording } from 'musicbrainz/types';

	interface Props {
		item: MediaItem;
		metadata?: DisplayMusicBrainzRecording | null;
		loading?: boolean;
		selected?: boolean;
		onselect?: (item: MediaItem) => void;
	}

	let { item, metadata = null, loading = false, selected = false, onselect }: Props = $props();
</script>

<MediaCardBase {item} imageUrl={metadata?.coverArtUrl ?? null} imageAlt={metadata?.title ?? item.name} {loading} {selected} onclick={() => onselect?.(item)}>
	{#if metadata}
		<p class="truncate text-xs font-medium" title={metadata.title}>{metadata.title}</p>
		<p class="truncate text-xs opacity-60">{metadata.artistCredits}</p>
		{#if metadata.duration}
			<p class="text-xs opacity-40">{metadata.duration}</p>
		{/if}
	{:else}
		<p class="truncate text-xs opacity-60" title={item.path}>{item.path}</p>
	{/if}
</MediaCardBase>
