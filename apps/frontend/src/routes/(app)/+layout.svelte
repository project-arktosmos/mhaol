<script lang="ts">
	import { onMount, onDestroy, setContext } from 'svelte';
	import { playerService } from 'ui-lib/services/player.service';
	import { identityService } from 'ui-lib/services/identity.service';
	import { torrentService } from 'ui-lib/services/torrent.service';
	import { signalingChatService } from 'ui-lib/services/signaling-chat.service';
	import { DEFAULT_SIGNALING_URL } from 'ui-lib/lib/api-base';
	import { invalidateAll } from '$app/navigation';

	import { youtubeService } from 'ui-lib/services/youtube.service';
	import { youtubeLibraryService } from 'ui-lib/services/youtube-library.service';
	import SmartSearchToast from 'ui-lib/components/llm/SmartSearchToast.svelte';
	import { smartSearchService } from 'ui-lib/services/smart-search.service';
	import { fetchRaw, resolveApiUrl } from 'ui-lib/transport/fetch-helpers';
	import { setImageBaseUrl } from 'addons/tmdb/transform';
	import { setCoverArtBaseUrl, setArtistImageBaseUrl } from 'addons/musicbrainz/transform';
	import { setRaImageBaseUrl } from 'addons/retroachievements/transform';
	import { rosterService } from 'ui-lib/services/roster.service';
	import { profileService } from 'ui-lib/services/profile.service';
	import { favoritesService } from 'ui-lib/services/favorites.service';
	import { connectionConfigService } from 'ui-lib/services/connection-config.service';
	import type { PassportData } from 'webrtc/types';

	const connConfig = connectionConfigService.get();
	if (!connConfig || connConfig.transportMode === 'http') {
		setImageBaseUrl(resolveApiUrl('/api/tmdb/image'));
		setCoverArtBaseUrl(resolveApiUrl('/api/musicbrainz/cover'));
		setArtistImageBaseUrl(resolveApiUrl('/api/musicbrainz/artist-image'));
		setRaImageBaseUrl(resolveApiUrl('/api/retroachievements/image'));
	}
	import PlayerOverlay from 'ui-lib/components/player/PlayerOverlay.svelte';
	import type { SmartSearchTorrentResult } from 'ui-lib/types/smart-search.type';
	import type { PlayableFile } from 'ui-lib/types/player.type';

	let { children } = $props();

	type BrowseViewMode = 'poster' | 'backdrop' | 'table';
	let browseViewModeValue = $state<BrowseViewMode>('poster');
	setContext('browseViewMode', {
		get value() {
			return browseViewModeValue;
		},
		set(mode: BrowseViewMode) {
			browseViewModeValue = mode;
		}
	});

	async function handleSmartSearchStream(candidate: SmartSearchTorrentResult) {
		smartSearchService.hide();
		const infoHash = await smartSearchService.startStream(candidate);
		if (!infoHash) return;
		invalidateAll();
		playerService.setDisplayMode('sidebar');

		let ready = false;
		const unsubscribe = torrentService.state.subscribe(() => {
			if (!ready) return;
			const torrent = torrentService.findByHash(infoHash);
			if (!torrent) return;

			smartSearchService.updateStreamingProgress(torrent.progress);

			if (torrent.progress >= 1.0 || torrent.state === 'seeding') {
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
					completedAt: ''
				};
				playerService.play(file);
			}
		});
		ready = true;
	}

	async function connectSignaling(): Promise<void> {
		const res = await fetchRaw('/api/signaling/client-identity');
		if (!res.ok) return;
		const data = await res.json();
		const passport: PassportData = data.passport;
		const serverRoom: string = data.serverRoom;

		const signFn = async (msg: string) => {
			const parts = msg.match(/^partykit-auth:(.+):(\d+)$/);
			if (!parts) throw new Error('Invalid auth message format');
			const authRes = await fetchRaw(
				`/api/signaling/auth?room_id=${encodeURIComponent(parts[1])}&timestamp=${parts[2]}`
			);
			if (!authRes.ok) throw new Error(`Auth signing failed: HTTP ${authRes.status}`);
			const authData = await authRes.json();
			return authData.signature;
		};

		await signalingChatService.connectToRoom(DEFAULT_SIGNALING_URL, serverRoom, passport, signFn);
	}

	const profileStore = profileService.state;
	let favoritesInitialized = false;

	$effect(() => {
		const wallet = $profileStore.local.wallet;
		if (wallet && !favoritesInitialized) {
			favoritesInitialized = true;
			favoritesService.initialize(wallet);
		}
	});

	onMount(() => {
		rosterService.initialize('api');
		profileService.initialize();
		youtubeService.initialize();
		youtubeLibraryService.initialize();
		torrentService.initialize('server');

		playerService.initialize();
		identityService.initialize();
		if (connConfig?.transportMode === 'webrtc') {
			connectSignaling().catch(() => {});
		}
	});

	onDestroy(() => {
		playerService.destroy();
		torrentService.destroy();
		signalingChatService.destroy();
		rosterService.destroy();
	});
</script>

{@render children?.()}

<PlayerOverlay />
<SmartSearchToast onlibrarychange={() => invalidateAll()} onstream={handleSmartSearchStream} />
