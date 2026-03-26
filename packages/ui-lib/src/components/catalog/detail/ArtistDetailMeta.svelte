<script lang="ts">
	import type { CatalogArtist } from 'ui-lib/types/catalog.type';

	interface Props {
		item: CatalogArtist;
	}

	let { item }: Props = $props();

	let type = $derived(item.metadata.type);
	let country = $derived(item.metadata.country);
	let disambiguation = $derived(item.metadata.disambiguation);
	let beginYear = $derived(item.metadata.beginYear);
	let endYear = $derived(item.metadata.endYear);
	let ended = $derived(item.metadata.ended);
	let tags = $derived(item.metadata.tags);
</script>

<div class="flex flex-col gap-3">
	{#if disambiguation}
		<p class="text-sm italic opacity-60">{disambiguation}</p>
	{/if}

	<div class="grid grid-cols-2 gap-2 text-sm">
		{#if type}
			<div>
				<span class="opacity-50">Type:</span>
				<span class="font-medium">{type}</span>
			</div>
		{/if}
		{#if country}
			<div>
				<span class="opacity-50">Country:</span>
				<span class="font-medium">{country}</span>
			</div>
		{/if}
		{#if beginYear}
			<div>
				<span class="opacity-50">Active:</span>
				<span class="font-medium">{beginYear}{ended && endYear ? ` – ${endYear}` : ended ? ' – dissolved' : ' – present'}</span>
			</div>
		{/if}
	</div>

	{#if tags.length > 0}
		<div>
			<h3 class="mb-1 text-xs font-semibold tracking-wide uppercase opacity-50">Tags</h3>
			<div class="flex flex-wrap gap-1">
				{#each tags.slice(0, 15) as tag}
					<span class="badge badge-outline badge-xs">{tag}</span>
				{/each}
			</div>
		</div>
	{/if}
</div>
