<script lang="ts">
	import '../css/app.css';
	import '$services/i18n';
	import { onMount } from 'svelte';
	import { themeService } from '$services/theme.service';
	import Navbar from '$components/landing/Navbar.svelte';

	let { children } = $props();

	onMount(() => {
		themeService.initialize();
	});
</script>

<div class="min-h-screen bg-base-100">
	<Navbar />
	{@render children?.()}
</div>
