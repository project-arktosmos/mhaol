<script lang="ts">
	import type { Snippet } from 'svelte';
	import classNames from 'classnames';
	import { addonKind } from 'cloud-ui';
	import { base } from '$app/paths';
	import FirkinCard from '$components/firkins/FirkinCard.svelte';
	import { getCachedImageUrl } from '$services/image-cache.service';
	import { movieTvViewModeService } from '$services/movie-tv-view-mode.service';
	import type { CloudFirkin } from '$types/firkin.type';

	interface Props {
		firkins: CloudFirkin[];
		collapsed?: boolean;
		collapsedCount?: number;
		moreHref?: string;
		emptyMessage?: string;
		hrefBuilder?: (firkin: CloudFirkin) => string;
		actions?: Snippet<[CloudFirkin]>;
		progressFor?: (firkin: CloudFirkin) => number | null;
	}

	let {
		firkins,
		collapsed = true,
		collapsedCount = 6,
		moreHref,
		emptyMessage = 'No firkins yet.',
		hrefBuilder,
		actions,
		progressFor
	}: Props = $props();

	const PREVIEW_COUNT = 4;

	// Mirror FirkinCard's per-kind shape on the "More" cell so it adopts the
	// same height as the surrounding cards regardless of view mode. The grid
	// is single-kind (one addon per page), so the first firkin's kind drives
	// the whole row.
	const viewModeStore = movieTvViewModeService.store;
	const referenceKind = $derived(firkins.length > 0 ? addonKind(firkins[0].addon) : null);
	const useLandscape = $derived(
		(referenceKind === 'movie' || referenceKind === 'tv show') &&
			$viewModeStore.mode === 'landscapes'
	);
	const moreFigureAspect = $derived(
		referenceKind === 'album'
			? 'aspect-square'
			: referenceKind === 'youtube video' || useLandscape
				? 'aspect-video'
				: 'aspect-[2/3]'
	);
	const moreHasStrip = $derived(
		referenceKind === 'album' || referenceKind === 'youtube video' || useLandscape
	);

	// Landscape cards are wider, so we drop from 7 cols (6 firkins + More) to
	// 5 cols (4 firkins + More) to keep tile widths in a similar range.
	const effectiveCollapsedCount = $derived(useLandscape ? 4 : collapsedCount);
	const gridColsClass = $derived(useLandscape ? 'grid-cols-5' : 'grid-cols-7');

	const visibleFirkins = $derived<CloudFirkin[]>(
		collapsed ? firkins.slice(0, effectiveCollapsedCount) : firkins
	);
	const hiddenCount = $derived(Math.max(0, firkins.length - effectiveCollapsedCount));
	const showMoreCell = $derived(collapsed && hiddenCount > 0 && !!moreHref);

	// The next four firkins after the visible slice — rendered as a 2x2 thumb
	// preview inside the "More" cell so the link visually represents what the
	// user is about to navigate into.
	const previewFirkins = $derived<CloudFirkin[]>(
		showMoreCell
			? firkins.slice(effectiveCollapsedCount, effectiveCollapsedCount + PREVIEW_COUNT)
			: []
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
	<div class={classNames('grid gap-4', gridColsClass)}>
		{#each visibleFirkins as doc (doc.id)}
			{@const progress = progressFor ? progressFor(doc) : null}
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
				{#if progress !== null && progress > 0}
					<div
						class="pointer-events-none absolute right-0 bottom-0 left-0 h-1 overflow-hidden bg-base-100/40"
					>
						<div class="h-full bg-primary" style="width: {Math.min(1, progress) * 100}%;"></div>
					</div>
				{/if}
				{#if actions}
					{@render actions(doc)}
				{/if}
			</div>
		{/each}
		{#if showMoreCell && moreHref}
			<article class="card w-full overflow-hidden rounded-md bg-base-200 shadow-sm">
				<figure class={classNames('relative w-full overflow-hidden bg-base-300', moreFigureAspect)}>
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
				</figure>
				{#if moreHasStrip}
					<div class="px-3 py-2" aria-hidden="true">
						<div class="text-sm font-semibold">&nbsp;</div>
						<div class="text-xs">&nbsp;</div>
					</div>
				{/if}
			</article>
		{/if}
	</div>
{/if}
