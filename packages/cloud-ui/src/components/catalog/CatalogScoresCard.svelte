<script lang="ts">
	interface Review {
		label: string;
		score: number;
		maxScore: number;
		voteCount?: number;
	}

	interface Props {
		reviews?: Review[];
	}

	let { reviews = [] }: Props = $props();

	function reviewPercent(review: Review): number | null {
		if (!Number.isFinite(review.maxScore) || review.maxScore <= 0) return null;
		const ratio = review.score / review.maxScore;
		if (!Number.isFinite(ratio)) return null;
		return Math.max(0, Math.min(100, ratio * 100));
	}

	function formatPercent(value: number): string {
		return `${Math.round(value)}%`;
	}

	const reviewPercents = $derived(
		reviews.map((r) => reviewPercent(r)).filter((v): v is number => typeof v === 'number')
	);
	const averagePercent = $derived(
		reviewPercents.length > 0
			? reviewPercents.reduce((sum, v) => sum + v, 0) / reviewPercents.length
			: null
	);
</script>

{#if reviews.length > 0}
	<div class="flex flex-col gap-2">
		<h2 class="text-sm font-semibold text-base-content/70 uppercase">Scores</h2>
		<table class="table table-xs">
			<tbody>
				{#each reviews as review (review.label)}
					{@const pct = reviewPercent(review)}
					<tr>
						<td class="font-semibold">{review.label}</td>
						<td class="text-right font-mono">
							{#if pct !== null}
								{formatPercent(pct)}
							{:else}
								—
							{/if}
						</td>
					</tr>
				{/each}
				{#if averagePercent !== null && reviewPercents.length > 1}
					<tr class="border-t-2 border-base-content/20 font-semibold">
						<td>Average</td>
						<td class="text-right font-mono">{formatPercent(averagePercent)}</td>
					</tr>
				{/if}
			</tbody>
		</table>
	</div>
{/if}
