<script lang="ts">
	import { base } from '$app/paths';
	import ThemeToggle from '$components/core/ThemeToggle.svelte';

	let { navItems = [] }: { navItems?: { href: string; label: string; classes?: string }[] } =
		$props();
</script>

<nav class="navbar sticky top-0 z-50 bg-base-100 shadow-sm">
	<div class="flex-1">
		<a href="{base}/" class="btn text-xl font-bold btn-ghost"
			>Mhaol <span class="text-primary">Media</span></a
		>
	</div>
	<div class="flex-none gap-2">
		{#if navItems}
			{#each navItems as item}
				<a href="{base}{item.href}" class="btn btn-sm {item.classes ?? ''}">{item.label}</a>
			{/each}
		{/if}
		<ThemeToggle />
	</div>
</nav>
