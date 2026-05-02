<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		target: HTMLElement | null | undefined;
		children: Snippet;
	}

	let { target, children }: Props = $props();
	let wrapper: HTMLDivElement | undefined = $state();

	$effect(() => {
		if (!target || !wrapper) return;
		target.appendChild(wrapper);
		return () => {
			if (wrapper && wrapper.parentNode) {
				wrapper.parentNode.removeChild(wrapper);
			}
		};
	});
</script>

<div bind:this={wrapper} style="display: contents">
	{@render children()}
</div>
