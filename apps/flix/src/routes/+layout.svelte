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
	import TorrentModalContent from 'ui-lib/components/torrent/TorrentModalContent.svelte';
	import DownloadsModalContent from 'ui-lib/components/downloads/DownloadsModalContent.svelte';
	import SignalingModalContent from 'ui-lib/components/signaling/SignalingModalContent.svelte';
	import IdentityModalContent from 'ui-lib/components/identity/IdentityModalContent.svelte';
	import VideoSettingsModalContent from 'ui-lib/components/settings/VideoSettingsModalContent.svelte';
	import ShareModalContent from 'ui-lib/components/share/ShareModalContent.svelte';
	import AddonsModalContent from 'ui-lib/components/addons/AddonsModalContent.svelte';
	import PluginsModalContent from 'ui-lib/components/plugins/PluginsModalContent.svelte';
	import { invalidateAll } from '$app/navigation';
	import SmartSearchToast from 'ui-lib/components/llm/SmartSearchToast.svelte';
	import { smartSearchService } from 'frontend/services/smart-search.service';
	import { apiUrl } from 'frontend/lib/api-base';
	import type { SmartSearchTorrentResult } from 'frontend/types/smart-search.type';
	import type { PlayableFile } from 'frontend/types/player.type';

	let { children } = $props();

	const playerState = playerService.state;

	const navItems = [
		{ id: 'torrent', label: 'Torrent', classes: 'btn-primary' },
		{ id: 'downloads', label: 'Downloads', classes: 'btn-secondary' },
		{ id: 'signaling', label: 'Signaling' },
		{ id: 'identity', label: 'Identity' },
		{ id: 'plugins', label: 'Plugins' },
		{ id: 'addons', label: 'Addons' },
		{ id: 'share', label: 'Share' },
		{ id: 'settings', label: 'Settings' }
	];

	const modals = {
		torrent: { component: TorrentModalContent, maxWidth: 'max-w-5xl' },
		downloads: { component: DownloadsModalContent, maxWidth: 'max-w-5xl' },
		signaling: { component: SignalingModalContent, maxWidth: 'max-w-5xl' },
		identity: { component: IdentityModalContent, maxWidth: 'max-w-3xl' },
		plugins: { component: PluginsModalContent, maxWidth: 'max-w-4xl' },
		addons: { component: AddonsModalContent, maxWidth: 'max-w-4xl' },
		share: { component: ShareModalContent, maxWidth: 'max-w-md' },
		settings: { component: VideoSettingsModalContent, maxWidth: 'max-w-2xl' },
	};

	async function handleSmartSearchStream(candidate: SmartSearchTorrentResult) {
		smartSearchService.hide();
		const infoHash = await smartSearchService.startStream(candidate);
		if (!infoHash) return;
		invalidateAll();

		const unsubscribe = torrentService.state.subscribe((state) => {
			const torrent = state.torrents.find((t) => t.infoHash === infoHash);
			if (!torrent) return;

			smartSearchService.updateStreamingProgress(torrent.progress);

			if (torrent.progress >= 0.02 || torrent.state === 'seeding') {
				unsubscribe();
				smartSearchService.clearStreaming();

				const file: PlayableFile = {
					id: `torrent:${infoHash}`,
					type: 'torrent',
					name: torrent.name,
					outputPath: torrent.outputPath ?? '',
					mode: 'video',
					format: null,
					videoFormat: null,
					thumbnailUrl: null,
					durationSeconds: null,
					size: torrent.size,
					completedAt: '',
					streamUrl: `/api/torrent/torrents/${infoHash}/stream`
				};
				playerService.playStream(file);
			}
		});
	}

	onMount(async () => {
		themeService.initialize('flix');
		await playerService.initialize();
		await identityService.initialize();
		torrentService.initialize('flix');
	});

	onDestroy(() => {
		playerService.destroy();
		torrentService.destroy();
	});
</script>

<div class="flex min-h-screen flex-col">
	<Navbar brand={{ label: 'Mhaol', highlight: 'Flix' }} items={navItems}>
		{#snippet end()}
			<ThemeToggle />
		{/snippet}
	</Navbar>
	<main class="flex min-w-0 flex-1 overflow-hidden">
		{@render children?.()}
	</main>
</div>

<ModalOutlet {modals} />
<SmartSearchToast onlibrarychange={() => invalidateAll()} onstream={handleSmartSearchStream} />
