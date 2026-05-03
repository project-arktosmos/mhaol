<script lang="ts">
	import '../css/app.css';
	import { onMount } from 'svelte';
	import { base } from '$app/paths';
	import ThemeToggle from '$components/core/ThemeToggle.svelte';

	let { children } = $props();

	onMount(() => {
		try {
			const stored = localStorage.getItem('mhaol-website:theme');
			if (stored !== 'dark' && stored !== 'light') {
				const prefersDark = window.matchMedia?.('(prefers-color-scheme: dark)').matches;
				document.documentElement.setAttribute('data-theme', prefersDark ? 'dark' : 'light');
			}
		} catch {
			/* ignore */
		}
	});
</script>

<div class="flex min-h-screen flex-col">
	<header class="sticky top-0 z-30 border-b border-base-300 bg-base-100/80 backdrop-blur">
		<div class="mx-auto flex w-full max-w-6xl items-center justify-between gap-4 px-4 py-3">
			<a href="{base}/" class="flex items-center gap-2 text-lg font-bold tracking-tight">
				<span class="text-base-content">Mhaol</span>
			</a>
			<nav class="hidden items-center gap-5 text-sm md:flex">
				<a href="{base}/#why" class="hover:text-primary">Why</a>
				<a href="{base}/#media" class="hover:text-primary">Media</a>
				<a href="{base}/#install" class="hover:text-primary">Install</a>
				<a href="{base}/#coming-up" class="hover:text-primary">Coming up</a>
			</nav>
			<div class="flex items-center gap-2">
				<ThemeToggle />
			</div>
		</div>
	</header>

	<main class="flex-1">
		{@render children?.()}
	</main>

	<footer class="border-t border-base-300 bg-base-200">
		<div
			class="mx-auto flex w-full max-w-6xl flex-col gap-3 px-4 py-6 text-sm text-base-content/70 md:flex-row md:items-center md:justify-between"
		>
			<div>Mhaol — self-hosted, content-addressed media cloud.</div>
			<div class="flex items-center gap-4">
				<a href="{base}/#install" class="hover:text-primary">Install</a>
				<a
					href="https://github.com/project-arktosmos/mhaol"
					class="hover:text-primary"
					rel="noopener">Source</a
				>
			</div>
		</div>
	</footer>
</div>
