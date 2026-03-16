<script lang="ts">
	import Modal from '$components/core/Modal.svelte';
	import { modalRouterService } from '$services/modal-router.service';
	import TorrentModalContent from '$components/torrent/TorrentModalContent.svelte';
	import DownloadsModalContent from '$components/downloads/DownloadsModalContent.svelte';
import SignalingModalContent from '$components/signaling/SignalingModalContent.svelte';
	import IdentityModalContent from '$components/identity/IdentityModalContent.svelte';
	import SettingsModalContent from '$components/settings/SettingsModalContent.svelte';
	import AddonsModalContent from '$components/addons/AddonsModalContent.svelte';
	import PluginsModalContent from '$components/plugins/PluginsModalContent.svelte';
	import LibraryModalContent from '$components/libraries/LibraryModalContent.svelte';

	const routerStore = modalRouterService.store;

	const MAX_WIDTHS: Record<string, string> = {
		torrent: 'max-w-5xl',
		downloads: 'max-w-5xl',
signaling: 'max-w-5xl',
		identity: 'max-w-3xl',
		plugins: 'max-w-4xl',
		addons: 'max-w-4xl',
		settings: 'max-w-2xl',
		libraries: 'max-w-5xl'
	};

	let activeId = $derived($routerStore.navbarModal);
	let maxWidth = $derived(activeId ? (MAX_WIDTHS[activeId] ?? 'max-w-lg') : 'max-w-lg');

	function handleClose() {
		modalRouterService.closeNavbar();
	}
</script>

<Modal open={!!activeId} {maxWidth} onclose={handleClose}>
	{#if activeId === 'torrent'}
		<TorrentModalContent />
	{:else if activeId === 'downloads'}
		<DownloadsModalContent />
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
	{:else if activeId === 'libraries'}
		<LibraryModalContent />
	{/if}
</Modal>
