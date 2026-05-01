<script lang="ts">
	import classNames from 'classnames';
	import type { RecommendationLabel } from '$types/recommendation-label.type';

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
			class={classNames('btn w-full btn-sm', {
				'btn-outline btn-primary': activeLabelId === label.id,
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
