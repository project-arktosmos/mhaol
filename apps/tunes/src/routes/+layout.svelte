<script lang="ts">
	import '../css/app.css';
	import 'frontend/services/i18n';
	import { onMount, onDestroy } from 'svelte';
	import { playerService } from 'frontend/services/player.service';
	import { identityService } from 'frontend/services/identity.service';
	import { torrentService } from 'frontend/services/torrent.service';
	import { themeService } from 'frontend/services/theme.service';
	import ThemeToggle from 'ui-lib/components/core/ThemeToggle.svelte';
	import Navbar from 'ui-lib/components/core/Navbar.svelte';
	import ModalOutlet from 'ui-lib/components/core/ModalOutlet.svelte';
	import SignalingModalContent from 'ui-lib/components/signaling/SignalingModalContent.svelte';
	import IdentityModalContent from 'ui-lib/components/identity/IdentityModalContent.svelte';
	import PluginsModalContent from 'ui-lib/components/plugins/PluginsModalContent.svelte';
	import AddonsModalContent from 'ui-lib/components/addons/AddonsModalContent.svelte';
	import TorrentModalContent from 'ui-lib/components/torrent/TorrentModalContent.svelte';
	import DownloadsModalContent from 'ui-lib/components/downloads/DownloadsModalContent.svelte';
	import SmartSearchToast from 'ui-lib/components/llm/SmartSearchToast.svelte';

	let { children } = $props();

	const navItems = [
		{ id: 'torrent', label: 'Torrent', classes: 'btn-primary' },
		{ id: 'downloads', label: 'Downloads', classes: 'btn-secondary' },
		{ id: 'signaling', label: 'Signaling' },
		{ id: 'identity', label: 'Identity' },
		{ id: 'plugins', label: 'Plugins' },
		{ id: 'addons', label: 'Addons' }
	];

	const modals = {
		torrent: { component: TorrentModalContent, maxWidth: 'max-w-5xl' },
		downloads: { component: DownloadsModalContent, maxWidth: 'max-w-5xl' },
		signaling: { component: SignalingModalContent, maxWidth: 'max-w-5xl' },
		identity: { component: IdentityModalContent, maxWidth: 'max-w-3xl' },
		plugins: { component: PluginsModalContent, maxWidth: 'max-w-4xl' },
		addons: { component: AddonsModalContent, maxWidth: 'max-w-4xl' }
	};

	onMount(async () => {
		themeService.initialize('tunes');
		await playerService.initialize();
		await identityService.initialize();
		torrentService.initialize('tunes');
	});

	onDestroy(() => {
		playerService.destroy();
		torrentService.destroy();
	});
</script>

<div class="flex min-h-screen flex-col">
	<Navbar brand={{ label: 'Mhaol', highlight: 'Tunes' }} items={navItems}>
		{#snippet end()}
			<ThemeToggle />
		{/snippet}
	</Navbar>
	<main class="flex min-w-0 flex-1 overflow-hidden">
		{@render children?.()}
	</main>
</div>

<ModalOutlet {modals} />
<SmartSearchToast />
