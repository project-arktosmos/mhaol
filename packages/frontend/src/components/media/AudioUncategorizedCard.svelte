<script lang="ts">
	import MediaCardBase from './MediaCardBase.svelte';
	import type { MediaItem } from '$types/media-card.type';
	import type { DisplayMusicBrainzRecording } from 'musicbrainz/types';

	interface Props {
		item: MediaItem;
		metadata?: DisplayMusicBrainzRecording | null;
		loading?: boolean;
		onlink?: (item: MediaItem, service: string) => void;
		onunlink?: (item: MediaItem, service: string) => void;
	}

	let { item, metadata = null, loading = false, onlink, onunlink }: Props = $props();

	let isLinked = $derived(!!item.links.musicbrainz);
</script>

<MediaCardBase {item} imageUrl={metadata?.coverArtUrl ?? null} imageAlt={metadata?.title ?? item.name} {loading}>
	{#if metadata}
		<p class="truncate text-xs font-medium" title={metadata.title}>{metadata.title}</p>
		<p class="truncate text-xs opacity-60">{metadata.artistCredits}</p>
		{#if metadata.duration}
			<p class="text-xs opacity-40">{metadata.duration}</p>
		{/if}
	{:else}
		<p class="truncate text-xs opacity-60" title={item.path}>{item.path}</p>
	{/if}
	{#snippet actions()}
		{#if isLinked}
			<button class="btn btn-ghost btn-xs" onclick={() => onunlink?.(item, 'musicbrainz')}>Unlink</button>
		{:else}
			<button class="btn btn-primary btn-xs" onclick={() => onlink?.(item, 'musicbrainz')}>Link metadata</button>
		{/if}
	{/snippet}
</MediaCardBase>
