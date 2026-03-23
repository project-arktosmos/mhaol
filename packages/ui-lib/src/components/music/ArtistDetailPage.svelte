<script lang="ts">
	import DetailPageLayout from 'ui-lib/components/core/DetailPageLayout.svelte';
	import type { DisplayMusicBrainzArtist } from 'addons/musicbrainz/types';

	interface Props {
		artist: DisplayMusicBrainzArtist;
		onback: () => void;
	}

	let { artist, onback }: Props = $props();

	let lifeSpan = $derived.by(() => {
		if (!artist.beginYear) return null;
		if (artist.ended && artist.endYear) return `${artist.beginYear} - ${artist.endYear}`;
		if (artist.beginYear) return `${artist.beginYear} - present`;
		return null;
	});
</script>

<DetailPageLayout>
	<button class="btn self-start btn-ghost btn-sm" onclick={onback}>
		<svg
			xmlns="http://www.w3.org/2000/svg"
			class="h-4 w-4"
			fill="none"
			viewBox="0 0 24 24"
			stroke="currentColor"
			stroke-width="2"
		>
			<path stroke-linecap="round" stroke-linejoin="round" d="M15 19l-7-7 7-7" />
		</svg>
		Back
	</button>

	<div class="flex items-center gap-4">
		<div
			class="flex h-24 w-24 shrink-0 items-center justify-center rounded-full bg-base-200 text-base-content/20"
		>
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
					stroke-width="1.5"
					d="M15.75 6a3.75 3.75 0 1 1-7.5 0 3.75 3.75 0 0 1 7.5 0ZM4.501 20.118a7.5 7.5 0 0 1 14.998 0A17.933 17.933 0 0 1 12 21.75c-2.676 0-5.216-.584-7.499-1.632Z"
				/>
			</svg>
		</div>
		<div class="min-w-0">
			<h1 class="text-xl font-bold">{artist.name}</h1>
			{#if artist.disambiguation}
				<p class="text-sm opacity-60">{artist.disambiguation}</p>
			{/if}
		</div>
	</div>

	<div class="flex flex-col gap-1.5">
		{#if artist.type}
			<div class="flex items-center gap-1 text-sm">
				<span class="opacity-40">Type:</span><span>{artist.type}</span>
			</div>
		{/if}
		{#if artist.country}
			<div class="flex items-center gap-1 text-sm">
				<span class="opacity-40">Country:</span><span>{artist.country}</span>
			</div>
		{/if}
		{#if lifeSpan}
			<div class="flex items-center gap-1 text-sm">
				<span class="opacity-40">Active:</span><span>{lifeSpan}</span>
			</div>
		{/if}
	</div>

	{#if artist.tags.length > 0}
		<div class="flex flex-wrap gap-1">
			{#each artist.tags as tag}
				<span class="badge badge-outline badge-sm">{tag}</span>
			{/each}
		</div>
	{/if}
</DetailPageLayout>
