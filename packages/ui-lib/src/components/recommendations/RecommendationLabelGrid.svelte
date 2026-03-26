<script lang="ts">
	import classNames from 'classnames';
	import type { RecommendationLabel } from 'ui-lib/types/recommendation-label.type';

	interface Props {
		labels: RecommendationLabel[];
		activeLabelId: string | null;
		loading?: boolean;
		onlabelclick?: (labelId: string) => void;
	}

	let { labels, activeLabelId, loading = false, onlabelclick }: Props = $props();
</script>

<div class="flex flex-wrap gap-2">
	{#each labels as label (label.id)}
		<button
			class={classNames('btn btn-sm', {
				'btn-primary': activeLabelId === label.id,
				'btn-ghost': activeLabelId !== label.id,
				'btn-disabled': loading
			})}
			onclick={() => onlabelclick?.(label.id)}
			disabled={loading}
		>
			{label.emoji}
		</button>
	{/each}
</div>
