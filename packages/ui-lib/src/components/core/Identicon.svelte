<script lang="ts">
	import classNames from 'classnames';
	import { blo } from 'blo';

	interface Props {
		address: string;
		title?: string;
		classes?: string;
	}

	let { address, title, classes = '' }: Props = $props();

	let src = $derived(address ? blo(address as `0x${string}`) : null);
	let label = $derived(title ?? `Identity: ${address}`);
</script>

{#if src}
	<img
		{src}
		alt=""
		class={classNames('rounded-full', classes)}
		title={label}
		aria-label={label}
	/>
{/if}
