<script lang="ts">
	import type { Snippet } from 'svelte';
	import { base } from '$app/paths';
	import FirkinCard from '$components/firkins/FirkinCard.svelte';
	import type { CloudFirkin } from '$types/firkin.type';
	import { firkinsService, type Firkin } from '$lib/firkins.service';

	interface Props {
		firkinIds: string[];
		collapsed?: boolean;
		collapsedCount?: number;
		moreHref?: string;
		emptyMessage?: string;
		actions?: Snippet<[Firkin]>;
	}

	let {
		firkinIds,
		collapsed = true,
		collapsedCount = 6,
		moreHref,
		emptyMessage = 'No firkins yet.',
		actions
	}: Props = $props();

	const firkinsStore = firkinsService.state;

	const firkinsById = $derived(new Map($firkinsStore.firkins.map((d) => [d.id, d] as const)));
	const allFirkins = $derived<Firkin[]>(
		firkinIds.map((id) => firkinsById.get(id)).filter((d): d is Firkin => d !== undefined)
	);
	const visibleFirkins = $derived<Firkin[]>(
		collapsed ? allFirkins.slice(0, collapsedCount) : allFirkins
	);
	const hiddenCount = $derived(Math.max(0, allFirkins.length - collapsedCount));
	const showMoreCell = $derived(collapsed && hiddenCount > 0 && !!moreHref);
</script>

{#if allFirkins.length === 0}
	<p class="text-sm text-base-content/60">{emptyMessage}</p>
{:else}
	<div class="grid grid-cols-7 gap-4">
		{#each visibleFirkins as doc (doc.id)}
			<div class="relative">
				<a
					href={`${base}/catalog/${encodeURIComponent(doc.id)}`}
					class="block no-underline"
					onclick={(e) => {
						if ((e.target as HTMLElement).closest('button, summary')) {
							e.preventDefault();
						}
					}}
				>
					<FirkinCard firkin={doc as CloudFirkin} />
				</a>
				{#if actions}
					{@render actions(doc)}
				{/if}
			</div>
		{/each}
		{#if showMoreCell && moreHref}
			<a
				href={moreHref}
				class="flex h-full min-h-32 w-full flex-col items-center justify-center gap-1 rounded-md border border-dashed border-base-content/20 bg-base-200 text-sm font-medium no-underline transition-colors hover:bg-base-300"
			>
				<span>More</span>
				<span class="text-xs text-base-content/60">+{hiddenCount}</span>
			</a>
		{/if}
	</div>
{/if}
