<script lang="ts">
	import type { Snippet } from 'svelte';
	import { base } from '$app/paths';
	import FirkinCard from '$components/firkins/FirkinCard.svelte';
	import { getCachedImageUrl } from '$services/image-cache.service';
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

	const PREVIEW_COUNT = 4;

	const visibleFirkins = $derived<CloudFirkin[]>(
		collapsed ? firkins.slice(0, collapsedCount) : firkins
	);
	const hiddenCount = $derived(Math.max(0, firkins.length - collapsedCount));
	const showMoreCell = $derived(collapsed && hiddenCount > 0 && !!moreHref);

	// The next four firkins after the visible slice — rendered as a 2x2 thumb
	// preview inside the "More" cell so the link visually represents what the
	// user is about to navigate into.
	const previewFirkins = $derived<CloudFirkin[]>(
		showMoreCell ? firkins.slice(collapsedCount, collapsedCount + PREVIEW_COUNT) : []
	);
	let previewUrls = $state<(string | null)[]>([]);

	$effect(() => {
		const sources = previewFirkins.map((f) => f.images?.[0]?.url ?? null);
		let cancelled = false;
		void Promise.all(
			sources.map(async (url) => {
				if (!url) return null;
				try {
					return await getCachedImageUrl(url);
				} catch {
					return null;
				}
			})
		).then((urls) => {
			if (!cancelled) previewUrls = urls;
		});
		return () => {
			cancelled = true;
		};
	});

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
			<div class="relative h-full min-h-32 w-full overflow-hidden rounded-md bg-base-300">
				<div class="grid h-full grid-cols-2 grid-rows-2 gap-px">
					{#each Array.from({ length: PREVIEW_COUNT }, (_, i) => i) as i (i)}
						<div class="overflow-hidden bg-base-200">
							{#if previewUrls[i]}
								<img
									src={previewUrls[i]}
									alt=""
									class="h-full w-full object-cover"
									loading="lazy"
								/>
							{/if}
						</div>
					{/each}
				</div>
				<div class="absolute inset-0 flex items-center justify-center bg-base-300/60">
					<a
						href={moreHref}
						class="btn btn-primary"
						aria-label={`More — ${hiddenCount} additional`}
					>
						More
					</a>
				</div>
			</div>
		{/if}
	</div>
{/if}
