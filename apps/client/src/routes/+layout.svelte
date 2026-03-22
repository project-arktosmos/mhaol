<script lang="ts">
	import '../css/app.css';
	import { onMount } from 'svelte';
	import Navbar from 'ui-lib/components/core/Navbar.svelte';
	import ThemeToggle from 'ui-lib/components/core/ThemeToggle.svelte';
	import { themeService } from 'frontend/services/theme.service';

	let { children } = $props();

	onMount(() => {
		themeService.initialize();
	});

	const themeState = themeService.state;

	$effect(() => {
		document.documentElement.setAttribute('data-theme', $themeState.theme);
	});
</script>

<Navbar brand={{ label: 'Mhaol Client' }}>
	{#snippet end()}
		<ThemeToggle />
	{/snippet}
</Navbar>
<main class="p-4">
	{@render children()}
</main>
