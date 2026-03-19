<script lang="ts">
	import '../css/app.css';
	import 'frontend/services/i18n';
	import { onMount, onDestroy } from 'svelte';
	import { themeService } from 'frontend/services/theme.service';
	import { torrentService } from 'frontend/services/torrent.service';
	import { libraryService } from 'frontend/services/library.service';
	import Navbar from 'ui-lib/components/core/Navbar.svelte';
	import ModalOutlet from 'ui-lib/components/core/ModalOutlet.svelte';
	import TorrentModalContent from 'ui-lib/components/torrent/TorrentModalContent.svelte';
	import SettingsModalContent from 'ui-lib/components/settings/SettingsModalContent.svelte';

	let { children } = $props();

	const themeStore = themeService.store;

	const navItems = [
		{ id: 'torrent', label: 'Torrent' },
		{ id: 'settings', label: 'Settings' }
	];

	const modals = {
		torrent: { component: TorrentModalContent, maxWidth: 'max-w-5xl' },
		settings: { component: SettingsModalContent, maxWidth: 'max-w-2xl' }
	};

	onMount(async () => {
		await Promise.all([torrentService.initialize(), libraryService.initialize()]);
	});

	onDestroy(() => {
		torrentService.destroy();
	});

	$effect(() => {
		document.documentElement.setAttribute('data-theme', $themeStore.theme);
	});
</script>

<div class="flex min-h-screen flex-col">
	<Navbar brand={{ label: 'Mhaol', highlight: 'Torrent' }} items={navItems} />
	<main class="min-w-0 flex-1">
		{@render children?.()}
	</main>
</div>

<ModalOutlet {modals} />
