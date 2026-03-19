<script lang="ts">
	import Modal from 'frontend/components/core/Modal.svelte';
	import { modalRouterService } from 'frontend/services/modal-router.service';
	import TorrentModalContent from 'frontend/components/torrent/TorrentModalContent.svelte';
	import DownloadsModalContent from 'frontend/components/downloads/DownloadsModalContent.svelte';
	import SignalingModalContent from 'frontend/components/signaling/SignalingModalContent.svelte';
	import IdentityModalContent from 'frontend/components/identity/IdentityModalContent.svelte';
	import SettingsModalContent from 'frontend/components/settings/SettingsModalContent.svelte';
	import AddonsModalContent from 'frontend/components/addons/AddonsModalContent.svelte';
	import PluginsModalContent from 'frontend/components/plugins/PluginsModalContent.svelte';
	import JackettModalContent from 'frontend/components/jackett/JackettModalContent.svelte';
	import LibraryModalContent from 'frontend/components/libraries/LibraryModalContent.svelte';
	import PeerLibrariesModalContent from 'frontend/components/peer-libraries/PeerLibrariesModalContent.svelte';
	import LlmModalContent from 'frontend/components/llm/LlmModalContent.svelte';

	const routerStore = modalRouterService.store;

	const MAX_WIDTHS: Record<string, string> = {
		torrent: 'max-w-5xl',
		downloads: 'max-w-5xl',
		jackett: 'max-w-5xl',
		signaling: 'max-w-5xl',
		'peer-libraries': 'max-w-5xl',
		identity: 'max-w-3xl',
		plugins: 'max-w-4xl',
		addons: 'max-w-4xl',
		settings: 'max-w-2xl',
		libraries: 'max-w-5xl',
		llm: 'max-w-6xl'
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
	{:else if activeId === 'jackett'}
		<JackettModalContent />
	{:else if activeId === 'downloads'}
		<DownloadsModalContent />
	{:else if activeId === 'signaling'}
		<SignalingModalContent />
	{:else if activeId === 'peer-libraries'}
		<PeerLibrariesModalContent />
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
	{:else if activeId === 'llm'}
		<LlmModalContent />
	{/if}
</Modal>
