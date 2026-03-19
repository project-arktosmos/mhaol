<script lang="ts">
	import '../css/app.css';
	import 'frontend/services/i18n';
	import { onMount, onDestroy } from 'svelte';
	import { themeService } from 'frontend/services/theme.service';
	import { torrentService } from 'frontend/services/torrent.service';
	import Navbar from 'ui-lib/components/core/Navbar.svelte';
	import ModalOutlet from 'ui-lib/components/core/ModalOutlet.svelte';
	import TorrentDownloadSettings from 'ui-lib/components/torrent/TorrentDownloadSettings.svelte';

	let { children } = $props();

	const themeStore = themeService.store;

	const navItems = [{ id: 'downloads', label: 'Downloads' }];

	const modals = {
		downloads: { component: TorrentDownloadSettings, maxWidth: 'max-w-lg' }
	};

	onMount(async () => {
		await torrentService.initialize();
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
