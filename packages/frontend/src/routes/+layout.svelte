<script lang="ts">
	import '../css/app.css';
	import Identicon from '$components/core/Identicon.svelte';
	import Navbar from '$components/core/Navbar.svelte';
	import ThemeToggle from '$components/core/ThemeToggle.svelte';
	import ToastOutlet from '$components/core/ToastOutlet.svelte';
	import NavbarAudioPlayer from '$components/player/NavbarAudioPlayer.svelte';
	import NavbarLyricsPanel from '$components/player/NavbarLyricsPanel.svelte';
	import NavbarPlaylistPanel from '$components/player/NavbarPlaylistPanel.svelte';
	import FirkinTooltip from '$components/firkins/FirkinTooltip.svelte';
	import SearchResultsPanel from '$components/catalog/SearchResultsPanel.svelte';
	import ArtistsModal from '$components/artists/ArtistsModal.svelte';
	import ConsumptionModal from '$components/consumption/ConsumptionModal.svelte';
	import DiskModal from '$components/disk/DiskModal.svelte';
	import HealthModal from '$components/health/HealthModal.svelte';
	import IpfsModal from '$components/ipfs/IpfsModal.svelte';
	import LibrariesModal from '$components/libraries/LibrariesModal.svelte';
	import TorrentModal from '$components/torrent/TorrentModal.svelte';
	import { artistsModalService } from '$services/artists-modal.service';
	import { consumptionModalService } from '$services/consumption-modal.service';
	import { diskModalService } from '$services/disk-modal.service';
	import { healthModalService } from '$services/health-modal.service';
	import { ipfsModalService } from '$services/ipfs-modal.service';
	import { librariesModalService } from '$services/libraries-modal.service';
	import { torrentModalService } from '$services/torrent-modal.service';
	import { playerService } from '$services/player.service';
	import { restorePlayerFromSnapshot } from '$lib/youtube-match.service';
	import { themeService } from '$services/theme.service';
	import { setBrowserImageCacheResolver } from '$services/image-cache.service';
	import { cachedImageUrl } from '$lib/image-cache';
	import { mediaTrackerService } from '$lib/media-tracker.service';
	import { userIdentityService } from '$lib/user-identity.service';
	import { installFetchInterceptor } from '$lib/install-fetch-interceptor';
	import { onMount, onDestroy } from 'svelte';

	installFetchInterceptor();
	import { base } from '$app/paths';

	setBrowserImageCacheResolver(cachedImageUrl);

	let { children } = $props();

	const playerState = playerService.state;
	const playerDisplayMode = playerService.displayMode;
	const identityState = userIdentityService.state;

	onMount(() => {
		themeService.initialize('flix');
		// Rehydrate the navbar player synchronously *before* `initialize` so the
		// snapshot persister sees the restored state on its first flush instead
		// of an empty placeholder. The async tail of `restorePlayerFromSnapshot`
		// (YouTube URL re-resolve) doesn't need to block the rest of init.
		void restorePlayerFromSnapshot();
		playerService.initialize();
		userIdentityService.initialize();
		mediaTrackerService.initialize();
	});

	onDestroy(() => {
		playerService.destroy();
	});
</script>

<div class="flex h-screen flex-col">
	<Navbar brand={{ label: 'Mhaol', highlight: 'Cloud' }} classes="!bg-base-300">
		{#snippet center()}
			<div class="flex flex-wrap items-center gap-2">
				<button
					type="button"
					class="btn btn-outline btn-sm"
					onclick={() => artistsModalService.open()}
					title="Browse content-addressed artist records"
				>
					Artists
				</button>
				<button
					type="button"
					class="btn btn-outline btn-sm"
					onclick={() => consumptionModalService.open()}
					title="Show playback time per firkin"
				>
					Consumption
				</button>
				<button
					type="button"
					class="btn btn-outline btn-sm"
					onclick={() => diskModalService.open()}
					title="Show host volumes and the data-root breakdown"
				>
					Disk
				</button>
				<button
					type="button"
					class="btn btn-outline btn-sm"
					onclick={() => healthModalService.open()}
					title="Live health of this Mhaol cloud node"
				>
					Health
				</button>
				<button
					type="button"
					class="btn btn-outline btn-sm"
					onclick={() => ipfsModalService.open()}
					title="Show IPFS pins recorded by the cloud"
				>
					IPFS
				</button>
				<button
					type="button"
					class="btn btn-outline btn-sm"
					onclick={() => librariesModalService.open()}
					title="Manage libraries on this machine"
				>
					Libraries
				</button>
				<button
					type="button"
					class="btn btn-outline btn-sm"
					onclick={() => torrentModalService.open()}
					title="Show the embedded torrent client status"
				>
					Torrent
				</button>
			</div>
		{/snippet}
		{#snippet end()}
			<div class="flex items-center gap-2">
				{#if $identityState.identity}
					<a
						href="{base}/profile"
						class="btn gap-2 font-mono normal-case btn-ghost btn-sm"
						title="Manage identity"
					>
						<Identicon
							address={$identityState.identity.address}
							title={`Identity: ${$identityState.identity.address}`}
							classes="h-5 w-5 shrink-0"
						/>
						{$identityState.identity.username}
					</a>
				{:else if $identityState.loading}
					<span class="loading loading-xs loading-spinner"></span>
				{/if}
				<ThemeToggle />
			</div>
		{/snippet}
	</Navbar>

	{#if $playerDisplayMode === 'navbar' && $playerState.currentFile}
		<div
			class="fixed right-2 bottom-2 z-40 flex w-96 max-w-[calc(100vw-1rem)] flex-col overflow-hidden rounded-lg border border-base-300 bg-base-100 shadow-lg"
		>
			<NavbarAudioPlayer />
			<NavbarLyricsPanel />
			<NavbarPlaylistPanel />
		</div>
	{/if}

	<main class="relative flex min-w-0 flex-1 overflow-hidden">
		<div class="relative flex min-w-0 flex-1 flex-col overflow-y-auto">
			{@render children?.()}
		</div>
		<SearchResultsPanel />
	</main>
</div>

<ToastOutlet />
<FirkinTooltip />
<ArtistsModal />
<ConsumptionModal />
<DiskModal />
<HealthModal />
<IpfsModal />
<LibrariesModal />
<TorrentModal />
