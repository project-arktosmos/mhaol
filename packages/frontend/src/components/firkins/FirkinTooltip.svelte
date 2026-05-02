<script lang="ts">
	import classNames from 'classnames';
	import { firkinTooltipService } from '$services/firkins/firkin-tooltip.svelte';
	import { getCachedImageUrl } from '$services/image-cache.service';
	import type { FirkinTooltipContent } from '$services/firkins/firkin-tooltip.svelte';

	const POINTER_OFFSET = 16;

	let resolvedImageUrl = $state<string | null>(null);

	function reviewPercent(review: { score: number; maxScore: number }): number | null {
		if (!Number.isFinite(review.maxScore) || review.maxScore <= 0) return null;
		const ratio = review.score / review.maxScore;
		if (!Number.isFinite(ratio)) return null;
		return Math.max(0, Math.min(100, ratio * 100));
	}

	function formatPercent(value: number): string {
		return `${Math.round(value)}%`;
	}

	function averagePercent(reviews: { score: number; maxScore: number }[]): number | null {
		const pcts: number[] = [];
		for (const r of reviews) {
			const p = reviewPercent(r);
			if (p !== null) pcts.push(p);
		}
		if (pcts.length === 0) return null;
		return pcts.reduce((sum, v) => sum + v, 0) / pcts.length;
	}

	function formatVotes(count: number): string {
		if (count >= 1_000_000) return `${(count / 1_000_000).toFixed(1)}M`;
		if (count >= 1000) return `${(count / 1000).toFixed(1)}k`;
		return `${count}`;
	}

	// Snapshot of the most recently shown content. Stays populated even after
	// `firkinTooltipService.content` flips back to null so the panel can fade
	// out using the previous content's data without re-evaluating against
	// null. Updated whenever a new content arrives — so hovering across
	// multiple firkin cards swaps content seamlessly.
	let displayContent = $state<FirkinTooltipContent | null>(null);
	$effect(() => {
		const c = firkinTooltipService.content;
		if (c) displayContent = c;
	});

	$effect(() => {
		const url = firkinTooltipService.content?.imageUrl ?? null;
		if (!url) {
			resolvedImageUrl = null;
			return;
		}
		let cancelled = false;
		getCachedImageUrl(url).then((u) => {
			if (!cancelled) resolvedImageUrl = u;
		});
		return () => {
			cancelled = true;
		};
	});

	let panelEl = $state<HTMLDivElement | null>(null);
	let viewport = $state({ width: 0, height: 0 });

	$effect(() => {
		const update = () => {
			viewport = { width: window.innerWidth, height: window.innerHeight };
		};
		update();
		window.addEventListener('resize', update);
		return () => window.removeEventListener('resize', update);
	});

	let panelSize = $state({ width: 0, height: 0 });

	$effect(() => {
		if (!panelEl) return;
		const observer = new ResizeObserver((entries) => {
			for (const entry of entries) {
				panelSize = {
					width: entry.contentRect.width,
					height: entry.contentRect.height
				};
			}
		});
		observer.observe(panelEl);
		return () => observer.disconnect();
	});

	let position = $derived.by(() => {
		const { x, y } = firkinTooltipService.pointer;
		const { width: vw, height: vh } = viewport;
		const { width: pw, height: ph } = panelSize;

		let left = x + POINTER_OFFSET;
		if (left + pw > vw - POINTER_OFFSET) {
			left = x - POINTER_OFFSET - pw;
		}
		left = Math.max(POINTER_OFFSET, Math.min(left, vw - pw - POINTER_OFFSET));

		let top = y + POINTER_OFFSET;
		if (top + ph > vh - POINTER_OFFSET) {
			top = y - POINTER_OFFSET - ph;
		}
		top = Math.max(POINTER_OFFSET, Math.min(top, vh - ph - POINTER_OFFSET));

		return { left, top };
	});

	const visible = $derived(firkinTooltipService.content !== null);
</script>

{#if displayContent}
	<div
		bind:this={panelEl}
		class={classNames(
			'pointer-events-none fixed z-[60] w-72 max-w-[calc(100vw-2rem)] overflow-hidden rounded-md border border-base-content/20 bg-base-100/95 shadow-xl backdrop-blur transition-opacity duration-150',
			visible ? 'opacity-100' : 'opacity-0'
		)}
		style="left: {position.left}px; top: {position.top}px;"
		role="tooltip"
	>
		{#if resolvedImageUrl}
			<img src={resolvedImageUrl} alt="" class="block h-auto w-full" loading="lazy" />
		{/if}
		<div class="px-4 py-3">
			<h3 class="text-base font-semibold [overflow-wrap:anywhere]">
				{displayContent.title}
			</h3>
			{#if displayContent.description}
				<p class="mt-2 text-xs [overflow-wrap:anywhere] whitespace-pre-wrap text-base-content/80">
					{displayContent.description}
				</p>
			{/if}
			{#if displayContent.reviews && displayContent.reviews.length > 0}
				{@const reviews = displayContent.reviews}
				{@const avg = averagePercent(reviews)}
				<table class="mt-3 w-full text-xs">
					<tbody>
						{#each reviews as review (review.label)}
							{@const pct = reviewPercent(review)}
							<tr class="border-t border-base-content/10">
								<th class="py-1 pr-2 text-left font-semibold">{review.label}</th>
								<td class="py-1 pr-2 text-right font-mono">
									{pct !== null ? formatPercent(pct) : '—'}
								</td>
								<td class="py-1 text-right font-mono text-base-content/60">
									{#if review.voteCount !== undefined}
										{formatVotes(review.voteCount)} votes
									{/if}
								</td>
							</tr>
						{/each}
						{#if avg !== null && reviews.length > 1}
							<tr class="border-t-2 border-base-content/30 bg-base-300/40 font-semibold">
								<th class="py-1 pr-2 text-left">Average</th>
								<td class="py-1 pr-2 text-right font-mono">{formatPercent(avg)}</td>
								<td></td>
							</tr>
						{/if}
					</tbody>
				</table>
			{/if}
		</div>
	</div>
{/if}
