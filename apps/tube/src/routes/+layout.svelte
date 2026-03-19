<script lang="ts">
	import { onMount } from 'svelte';
	import '../css/app.css';
	import '$services/i18n';
	import { themeService } from '$services/theme.service';
	import { youtubeService } from '$services/youtube.service';
	import { libraryService } from '$services/library.service';
	import Navbar from '$components/core/Navbar.svelte';
	import RightPanel from '$components/core/RightPanel.svelte';

	let { children } = $props();

	const themeStore = themeService.store;

	onMount(() => {
		youtubeService.initialize();
		libraryService.initialize();
	});

	$effect(() => {
		document.documentElement.setAttribute('data-theme', $themeStore.theme);
	});
</script>

<div class="flex h-screen flex-col overflow-hidden">
	<Navbar />
	<div class="flex min-h-0 flex-1">
		<main class="min-w-0 flex-1 overflow-y-auto bg-base-300">
			{@render children?.()}
		</main>
		<RightPanel />
	</div>
</div>
