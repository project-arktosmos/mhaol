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

<div class="grid grid-cols-4 gap-2">
	{#each labels as label (label.id)}
		<button
			class={classNames('btn btn-sm w-full', {
				'btn-primary btn-outline': activeLabelId === label.id,
				'btn-outline': activeLabelId !== label.id,
				'btn-disabled': loading
			})}
			onclick={() => onlabelclick?.(label.id)}
			disabled={loading}
		>
			{label.emoji}
		</button>
	{/each}
</div>
