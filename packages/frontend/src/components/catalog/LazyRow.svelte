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

<!--
	Once visible, the wrapper switches to `display: contents` so its children
	become direct flex items of the parent. That way a child that decides to
	render nothing (e.g. PopularGenreRow hiding a genre that can't fill the
	7-col row) doesn't leave the wrapper as an empty flex item that still
	takes a `gap-6` slot in the catalog page's column layout.
-->
<div
	bind:this={el}
	style:min-height={visible ? undefined : minHeight}
	style:display={visible ? 'contents' : undefined}
>
	{#if visible}
		{@render children()}
	{/if}
</div>
