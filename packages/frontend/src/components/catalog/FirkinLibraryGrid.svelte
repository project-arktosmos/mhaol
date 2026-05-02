<script lang="ts">
	import type { Snippet } from 'svelte';
	import { base } from '$app/paths';
	import FirkinCard from '$components/firkins/FirkinCard.svelte';
	import type { CloudFirkin } from '$types/firkin.type';

	interface Props {
		firkins: CloudFirkin[];
		collapsed?: boolean;
		collapsedCount?: number;
		moreHref?: string;
		emptyMessage?: string;
		hrefBuilder?: (firkin: CloudFirkin) => string;
		actions?: Snippet<[CloudFirkin]>;
	}

	let {
		firkins,
		collapsed = true,
		collapsedCount = 6,
		moreHref,
		emptyMessage = 'No firkins yet.',
		hrefBuilder,
		actions
	}: Props = $props();

	const visibleFirkins = $derived<CloudFirkin[]>(
		collapsed ? firkins.slice(0, collapsedCount) : firkins
	);
	const hiddenCount = $derived(Math.max(0, firkins.length - collapsedCount));
	const showMoreCell = $derived(collapsed && hiddenCount > 0 && !!moreHref);

	function defaultHref(firkin: CloudFirkin): string {
		return `${base}/catalog/${encodeURIComponent(firkin.id)}`;
	}
</script>

{#if firkins.length === 0}
	<p class="text-sm text-base-content/60">{emptyMessage}</p>
{:else}
	<div class="grid grid-cols-7 gap-4">
		{#each visibleFirkins as doc (doc.id)}
			<div class="relative">
				<a
					href={hrefBuilder ? hrefBuilder(doc) : defaultHref(doc)}
					class="block no-underline"
					onclick={(e) => {
						if ((e.target as HTMLElement).closest('button, summary')) {
							e.preventDefault();
						}
					}}
				>
					<FirkinCard firkin={doc} />
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
