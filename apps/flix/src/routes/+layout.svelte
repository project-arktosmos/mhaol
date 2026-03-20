<script lang="ts">
	import '../css/app.css';
	import 'frontend/services/i18n';
	import { onMount, onDestroy } from 'svelte';
	import { playerService } from 'frontend/services/player.service';
	import { identityService } from 'frontend/services/identity.service';
	import { torrentService } from 'frontend/services/torrent.service';
	import { themeService } from 'frontend/services/theme.service';
	import IdentitySidebar from 'ui-lib/components/core/IdentitySidebar.svelte';
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
	import PlayerVideo from 'ui-lib/components/player/PlayerVideo.svelte';
	import { smartSearchService } from 'frontend/services/smart-search.service';
	import { apiUrl } from 'frontend/lib/api-base';
	import type { SmartSearchTorrentResult } from 'frontend/types/smart-search.type';
	import type { PlayableFile } from 'frontend/types/player.type';

	let { children } = $props();

	const playerState = playerService.state;
	const playerDisplayMode = playerService.displayMode;

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
		themeService.initialize();
		await playerService.initialize();
		await identityService.initialize();
		torrentService.initialize();
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
	<div class="flex flex-1">
		<main class="flex min-w-0 flex-1 overflow-hidden">
			{@render children?.()}
		</main>
		<div class="hidden flex-col lg:flex">
			{#if $playerState.currentFile && $playerDisplayMode === 'sidebar'}
				<div class="w-85 border-l border-base-300 bg-base-200">
					<div class="flex items-center justify-between p-2">
						<p class="min-w-0 truncate text-xs font-semibold" title={$playerState.currentFile.name}>
							{$playerState.currentFile.name}
						</p>
						<div class="flex shrink-0 items-center gap-1">
							<button
								class="btn btn-square btn-ghost btn-xs"
								onclick={() => playerService.setDisplayMode('fullscreen')}
								aria-label="Fullscreen player"
								title="Fullscreen player"
							>
								<svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
									<path stroke-linecap="round" stroke-linejoin="round" d="M4 8V4h4M20 8V4h-4M4 16v4h4M20 16v4h-4" />
								</svg>
							</button>
							<button
								class="btn btn-square btn-ghost btn-xs"
								onclick={() => playerService.stop()}
								aria-label="Close player"
							>
								&times;
							</button>
						</div>
					</div>
					<PlayerVideo
						file={$playerState.currentFile}
						connectionState={$playerState.connectionState}
						positionSecs={$playerState.positionSecs}
						durationSecs={$playerState.durationSecs}
						streamUrl={$playerState.streamUrl}
						buffering={$playerState.buffering}
					/>
				</div>
			{/if}
			<IdentitySidebar classes="flex-1" />
		</div>
	</div>
</div>

<ModalOutlet {modals} />
<SmartSearchToast onlibrarychange={() => invalidateAll()} onstream={handleSmartSearchStream} />
