<script lang="ts">
	import Modal from 'frontend/components/core/Modal.svelte';
	import { modalRouterService } from 'frontend/services/modal-router.service';
	import YouTubeModalContent from 'frontend/components/youtube/YouTubeModalContent.svelte';
	import TorrentModalContent from 'frontend/components/torrent/TorrentModalContent.svelte';
	import DownloadsModalContent from 'frontend/components/downloads/DownloadsModalContent.svelte';
	import LibraryModalContent from 'frontend/components/libraries/LibraryModalContent.svelte';
	import SignalingModalContent from 'frontend/components/signaling/SignalingModalContent.svelte';
	import IdentityModalContent from 'frontend/components/identity/IdentityModalContent.svelte';
	import SettingsModalContent from 'frontend/components/settings/SettingsModalContent.svelte';
	import AddonsModalContent from 'frontend/components/addons/AddonsModalContent.svelte';
	import PluginsModalContent from 'frontend/components/plugins/PluginsModalContent.svelte';
	import YouTubeSearchModalContent from 'frontend/components/youtube-search/YouTubeSearchModalContent.svelte';

	const routerStore = modalRouterService.store;

	const MAX_WIDTHS: Record<string, string> = {
		youtube: 'max-w-6xl',
		'youtube-search': 'max-w-5xl',
		torrent: 'max-w-5xl',
		downloads: 'max-w-5xl',
		libraries: 'max-w-5xl',
		signaling: 'max-w-5xl',
		identity: 'max-w-3xl',
		plugins: 'max-w-4xl',
		addons: 'max-w-4xl',
		settings: 'max-w-2xl'
	};

	let activeId = $derived($routerStore.navbarModal);
	let maxWidth = $derived(activeId ? (MAX_WIDTHS[activeId] ?? 'max-w-lg') : 'max-w-lg');

	function handleClose() {
		modalRouterService.closeNavbar();
	}
</script>

<Modal open={!!activeId} {maxWidth} onclose={handleClose}>
	{#if activeId === 'youtube-search'}
		<YouTubeSearchModalContent />
	{:else if activeId === 'youtube'}
		<YouTubeModalContent />
	{:else if activeId === 'torrent'}
		<TorrentModalContent />
	{:else if activeId === 'downloads'}
		<DownloadsModalContent />
	{:else if activeId === 'libraries'}
		<LibraryModalContent />
	{:else if activeId === 'signaling'}
		<SignalingModalContent />
	{:else if activeId === 'identity'}
		<IdentityModalContent />
	{:else if activeId === 'settings'}
		<SettingsModalContent />
	{:else if activeId === 'addons'}
		<AddonsModalContent />
	{:else if activeId === 'plugins'}
		<PluginsModalContent />
	{/if}
</Modal>
