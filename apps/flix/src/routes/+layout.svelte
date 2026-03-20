<script lang="ts">
	import '../css/app.css';
	import 'frontend/services/i18n';
	import { onMount, onDestroy } from 'svelte';
	import { playerService } from 'frontend/services/player.service';
	import { identityService } from 'frontend/services/identity.service';
	import { peerLibraryService } from 'frontend/services/peer-library.service';
	import { torrentService } from 'frontend/services/torrent.service';
	import IdentitySidebar from 'ui-lib/components/core/IdentitySidebar.svelte';
	import Navbar from 'ui-lib/components/core/Navbar.svelte';
	import ModalOutlet from 'ui-lib/components/core/ModalOutlet.svelte';
	import TorrentModalContent from 'ui-lib/components/torrent/TorrentModalContent.svelte';
	import DownloadsModalContent from 'ui-lib/components/downloads/DownloadsModalContent.svelte';
	import SignalingModalContent from 'ui-lib/components/signaling/SignalingModalContent.svelte';
	import IdentityModalContent from 'ui-lib/components/identity/IdentityModalContent.svelte';
	import SettingsModalContent from 'ui-lib/components/settings/SettingsModalContent.svelte';
	import AddonsModalContent from 'ui-lib/components/addons/AddonsModalContent.svelte';
	import PluginsModalContent from 'ui-lib/components/plugins/PluginsModalContent.svelte';
	import JackettModalContent from 'ui-lib/components/jackett/JackettModalContent.svelte';
	import LibraryModalContent from 'ui-lib/components/libraries/LibraryModalContent.svelte';
	import PeerLibrariesModalContent from 'ui-lib/components/peer-libraries/PeerLibrariesModalContent.svelte';
	import LlmModalContent from 'ui-lib/components/llm/LlmModalContent.svelte';
	import { invalidateAll } from '$app/navigation';
	import SmartSearchToast from 'ui-lib/components/llm/SmartSearchToast.svelte';

	let { children } = $props();

	const navItems = [
		{ id: 'torrent', label: 'Torrent', classes: 'btn-primary' },
		{ id: 'jackett', label: 'Jackett', classes: 'btn-primary' },
		{ id: 'downloads', label: 'Downloads', classes: 'btn-secondary' },
		{ id: 'signaling', label: 'Signaling' },
		{ id: 'peer-libraries', label: 'Peers' },
		{ id: 'identity', label: 'Identity' },
		{ id: 'plugins', label: 'Plugins' },
		{ id: 'addons', label: 'Addons' },
		{ id: 'libraries', label: 'Libraries' },
		{ id: 'llm', label: 'LLM' },
		{ id: 'settings', label: 'Settings' }
	];

	const modals = {
		torrent: { component: TorrentModalContent, maxWidth: 'max-w-5xl' },
		jackett: { component: JackettModalContent, maxWidth: 'max-w-5xl' },
		downloads: { component: DownloadsModalContent, maxWidth: 'max-w-5xl' },
		signaling: { component: SignalingModalContent, maxWidth: 'max-w-5xl' },
		'peer-libraries': { component: PeerLibrariesModalContent, maxWidth: 'max-w-5xl' },
		identity: { component: IdentityModalContent, maxWidth: 'max-w-3xl' },
		plugins: { component: PluginsModalContent, maxWidth: 'max-w-4xl' },
		addons: { component: AddonsModalContent, maxWidth: 'max-w-4xl' },
		settings: { component: SettingsModalContent, maxWidth: 'max-w-2xl' },
		libraries: { component: LibraryModalContent, maxWidth: 'max-w-5xl' },
		llm: { component: LlmModalContent, maxWidth: 'max-w-6xl' }
	};

	onMount(async () => {
		await playerService.initialize();
		await identityService.initialize();
		peerLibraryService.initialize();
		torrentService.initialize();
	});

	onDestroy(() => {
		playerService.destroy();
		torrentService.destroy();
	});
</script>

<div class="flex min-h-screen flex-col">
	<Navbar brand={{ label: 'Mhaol', highlight: 'Flix' }} items={navItems} />
	<div class="flex flex-1">
		<main class="min-w-0 flex-1">
			{@render children?.()}
		</main>
		<IdentitySidebar />
	</div>
</div>

<ModalOutlet {modals} />
<SmartSearchToast onlibrarychange={() => invalidateAll()} />
