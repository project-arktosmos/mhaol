<script lang="ts">
	import type { CatalogAlbum } from 'ui-lib/types/catalog.type';

	interface Props {
		item: CatalogAlbum;
	}

	let { item }: Props = $props();

	let artistCredits = $derived(item.metadata.artistCredits);
	let primaryType = $derived(item.metadata.primaryType);
	let releases = $derived(item.metadata.releases);
	let firstRelease = $derived(releases[0] ?? null);
	let tracks = $derived(firstRelease?.tracks ?? []);
</script>

<div class="flex flex-col gap-3">
	<div class="text-sm">
		<span class="opacity-50">Artist:</span>
		<span class="font-medium">{artistCredits}</span>
	</div>

	{#if primaryType}
		<div class="text-sm">
			<span class="opacity-50">Type:</span>
			<span class="badge badge-ghost badge-sm">{primaryType}</span>
		</div>
	{/if}

	{#if firstRelease}
		<div class="grid grid-cols-2 gap-2 text-sm">
			{#if firstRelease.date}
				<div>
					<span class="opacity-50">Released:</span>
					<span class="font-medium">{firstRelease.date}</span>
				</div>
			{/if}
			{#if firstRelease.label}
				<div>
					<span class="opacity-50">Label:</span>
					<span class="font-medium">{firstRelease.label}</span>
				</div>
			{/if}
			{#if firstRelease.country}
				<div>
					<span class="opacity-50">Country:</span>
					<span class="font-medium">{firstRelease.country}</span>
				</div>
			{/if}
		</div>
	{/if}

	{#if tracks.length > 0}
		<div>
			<h3 class="mb-1 text-xs font-semibold tracking-wide uppercase opacity-50">
				Tracks ({tracks.length})
			</h3>
			<div class="flex flex-col">
				{#each tracks as track}
					<div
						class="flex items-center justify-between border-b border-base-200 py-1.5 text-sm last:border-0"
					>
						<div class="flex items-center gap-2">
							<span class="w-6 text-right text-xs opacity-40">{track.number}</span>
							<span>{track.title}</span>
						</div>
						{#if track.duration}
							<span class="text-xs opacity-50">{track.duration}</span>
						{/if}
					</div>
				{/each}
			</div>
		</div>
	{/if}
</div>
