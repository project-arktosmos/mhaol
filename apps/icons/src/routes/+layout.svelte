<script lang="ts">
	import '../css/app.css';
	import { themeService } from 'frontend/services/theme.service';
	import Navbar from 'ui-lib/components/core/Navbar.svelte';

	let { children } = $props();

	const themeStore = themeService.store;

	$effect(() => {
		document.documentElement.setAttribute('data-theme', $themeStore.theme);
	});
</script>

<div class="flex min-h-screen flex-col">
	<Navbar brand={{ label: 'Mhaol', highlight: 'Icons' }} items={[]} />
	<main class="min-w-0 flex-1">
		{@render children?.()}
	</main>
</div>
