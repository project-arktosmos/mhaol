<script lang="ts">
	import classNames from 'classnames';

	interface Props {
		tag: string;
		score: number;
		readonly?: boolean;
		onremove?: (tag: string) => void;
	}

	let { tag, score, readonly: isReadonly = false, onremove }: Props = $props();

	let badgeClass = $derived(
		score >= 0.1 ? 'badge-success' : score >= 0.03 ? 'badge-info' : 'badge-ghost'
	);

	let percentage = $derived(Math.round(score * 100));
</script>

<span class={classNames('badge badge-sm gap-1', badgeClass)} title={`${tag}: ${percentage}%`}>
	{tag}
	<span class="opacity-60">{percentage}%</span>
	{#if !isReadonly}
		<button
			class="ml-0.5 opacity-40 hover:opacity-100 cursor-pointer"
			onclick={() => onremove?.(tag)}
			title="Remove tag"
		>
			&times;
		</button>
	{/if}
</span>
