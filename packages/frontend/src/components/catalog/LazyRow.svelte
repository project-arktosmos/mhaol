<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		rootMargin?: string;
		minHeight?: string;
		children: Snippet;
	}

	let { rootMargin = '200px', minHeight = '18rem', children }: Props = $props();

	let visible = $state(false);
	let el = $state<HTMLDivElement | null>(null);

	$effect(() => {
		if (!el || visible) return;
		if (typeof IntersectionObserver === 'undefined') {
			visible = true;
			return;
		}
		const observer = new IntersectionObserver(
			(entries) => {
				for (const entry of entries) {
					if (entry.isIntersecting) {
						visible = true;
						observer.disconnect();
						break;
					}
				}
			},
			{ rootMargin }
		);
		observer.observe(el);
		return () => observer.disconnect();
	});
</script>

<div bind:this={el} style:min-height={visible ? undefined : minHeight}>
	{#if visible}
		{@render children()}
	{/if}
</div>
