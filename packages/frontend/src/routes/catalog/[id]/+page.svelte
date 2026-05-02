<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import { get } from 'svelte/store';
	import classNames from 'classnames';
	import FirkinArtistsSection from '$components/firkins/FirkinArtistsSection.svelte';
	import FirkinMetadataLookupModal, {
		type CatalogLookupItem
	} from '$components/firkins/FirkinMetadataLookupModal.svelte';
	import CatalogPageHeader from '$components/catalog/CatalogPageHeader.svelte';
	import { CatalogScoresCard } from 'cloud-ui';
	import CatalogDescriptionPanel from '$components/catalog/CatalogDescriptionPanel.svelte';
	import CatalogTvSeasonsCard from '$components/catalog/CatalogTvSeasonsCard.svelte';
	import CatalogTrailerPlayer from '$components/catalog/CatalogTrailerPlayer.svelte';
	import PlayerVideo from '$components/player/PlayerVideo.svelte';
	import CatalogTracksCard from '$components/catalog/CatalogTracksCard.svelte';
	import CatalogTorrentSearchCard from '$components/catalog/CatalogTorrentSearchCard.svelte';
	import CatalogTorrentAttachmentCard, {
		type AttachmentInfo,
		type DownloadAttachmentInfo
	} from '$components/catalog/CatalogTorrentAttachmentCard.svelte';
	import CatalogSubsLyricsCard from '$components/catalog/CatalogSubsLyricsCard.svelte';
	import CatalogRelatedCard from '$components/catalog/CatalogRelatedCard.svelte';
	import CatalogAlbumsByArtistCard from '$components/catalog/CatalogAlbumsByArtistCard.svelte';
	import CatalogChannelLatestCard from '$components/catalog/CatalogChannelLatestCard.svelte';
	import CatalogRelatedYoutubeCard from '$components/catalog/CatalogRelatedYoutubeCard.svelte';
	import CatalogFilesTable from '$components/catalog/CatalogFilesTable.svelte';
	import { firkinPlaybackService } from '$services/firkin-playback.service';
	import { firkinTorrentsService, infoHashFromMagnet } from '$services/firkin-torrents.service';
	import { playerService } from '$services/player.service';
	import type { CloudFirkin } from '$types/firkin.type';
	import type { PlayableFile, PlayableFileSubtitle } from '$types/player.type';
	import type { SubsLyricsItem } from '$types/subs-lyrics.type';
	import { matchSubsToTorrent } from '$utils/match-subs-to-torrent';
	import {
		firkinsService,
		addonKind,
		metadataSearchAddon,
		type Firkin,
		type FirkinAddon,
		type FileEntry
	} from '$lib/firkins.service';
	import { TrailerResolver } from '$services/catalog/trailer-resolver.svelte';
	import {
		TrackResolver,
		type AlbumProgressPayload,
		type AlbumDownloadProgressPayload
	} from '$services/catalog/track-resolver.svelte';
	import { TorrentSearch, startTorrentDownload } from '$services/catalog/torrent-search.svelte';
	import { SubsLyricsResolver } from '$services/catalog/subs-lyrics-resolver.svelte';
	import { parseTorrentSeasons, type TorrentResultItem } from '$lib/search.service';
	import {
		ingestRecommendations,
		type RecommendationIngestItem
	} from '$lib/recommendations.service';
	import type { CatalogItem } from '$lib/catalog.service';
	import { userIdentityService } from '$lib/user-identity.service';
	import { base } from '$app/paths';
	import { goto } from '$app/navigation';

	interface Props {
		data: { firkin: Firkin };
	}

	let { data }: Props = $props();
	// Local override so client-side mutations (bookmark flip, metadata
	// enrich, trailer/track persist, polling rollforward) trigger
	// reactivity. Props from `$props()` are read-only in Svelte 5 —
	// `data.firkin = updated` silently fails to propagate, so the
	// `bookmarked` flip required a page refresh to surface. The override
	// is dropped on navigation (a new loader payload arrives via
	// `data.firkin`) so the page always stays in sync with SvelteKit.
	let firkinOverride = $state<Firkin | null>(null);
	// Drop any in-page mutation override the moment SvelteKit hands us a
	// loader payload for a different firkin id — without this, navigating
	// from /catalog/A to /catalog/B (e.g. clicking a related card) would
	// keep rendering A because the `firkin` derivation prefers the
	// override over the freshly-loaded `data.firkin`.
	$effect.pre(() => {
		const incomingId = data.firkin.id;
		if (firkinOverride && firkinOverride.id !== incomingId) {
			firkinOverride = null;
		}
	});
	const firkin = $derived<Firkin>(firkinOverride ?? data.firkin);
	// `bookmarked` flips between two presentations of the same detail page:
	// non-bookmarked firkins (created by the catalog `/catalog/visit`
	// resolver) show only the Bookmark action and skip identity / version
	// history / files / torrent-search / IPFS-Torrent tabs — the same
	// surface the now-deleted `/catalog/virtual` page used to render. Once
	// the user clicks Bookmark, the same record gains the full detail
	// surface (Play / Find metadata / Delete, plus torrent search auto-fire).
	const isBookmarked = $derived(firkin.bookmarked !== false);
	let bookmarking = $state(false);
	let bookmarkError = $state<string | null>(null);

	const playerState = playerService.state;
	const playerDisplayMode = playerService.displayMode;
	const isInlinePlayingThisFirkin = $derived(
		$playerDisplayMode === 'inline' &&
			$playerState.firkinId === firkin.id &&
			Boolean($playerState.directStreamUrl)
	);

	type StreamSource = 'trailer' | 'ipfs' | 'torrent';
	let activeSource = $state<StreamSource>('trailer');

	onDestroy(() => {
		const state = get(playerService.state);
		const mode = get(playerService.displayMode);
		if (mode === 'inline' && state.firkinId === firkin.id) {
			void playerService.stop();
		}
	});

	// IPFS streaming runs the file through a GStreamer hlssink2 pipeline
	// (decodebin → x264 + AAC → HLS), so the source has to be real
	// video/audio. Library scans pin sibling files too — VobSub `.sub`
	// subtitles, ROM `.iso`s, sample/promo clips — and picking those
	// would either stall decodebin forever (no video/audio pad to link)
	// or stream the wrong thing. We filter to playable extensions, skip
	// obvious non-main files, and prefer `.mkv` so a BluRay rip's main
	// container wins over a tiny `ETRG.mp4` promo.
	const EXT_PRIORITY: Record<string, number> = {
		'.mkv': 0,
		'.mp4': 1,
		'.m4v': 2,
		'.mov': 3,
		'.webm': 4,
		'.avi': 5,
		'.ts': 6,
		'.m2ts': 7,
		'.mpg': 8,
		'.mpeg': 9,
		'.ogv': 10,
		'.wmv': 11,
		'.flv': 12,
		'.flac': 13,
		'.m4a': 14,
		'.mp3': 15,
		'.opus': 16,
		'.ogg': 17,
		'.wav': 18,
		'.aac': 19
	};
	const NON_MAIN_KEYWORDS = ['sample', 'trailer', 'promo', 'extras', 'behind', 'bonus'];
	function extOf(title: string): string | null {
		const lower = title.toLowerCase();
		const idx = lower.lastIndexOf('.');
		return idx >= 0 ? lower.slice(idx) : null;
	}
	function isPlayableMedia(title: string | undefined | null): boolean {
		if (!title) return false;
		const ext = extOf(title);
		return ext !== null && ext in EXT_PRIORITY;
	}
	function isMainContent(title: string): boolean {
		const lower = title.toLowerCase();
		return !NON_MAIN_KEYWORDS.some((kw) => lower.includes(kw));
	}
	const playableIpfsFiles = $derived.by(() => {
		const matched = firkin.files.filter((f) => f.type === 'ipfs' && isPlayableMedia(f.title));
		const main = matched.filter((f) => isMainContent(f.title ?? ''));
		const pool = main.length > 0 ? main : matched;
		return [...pool].sort((a, b) => {
			const ae = extOf(a.title ?? '') ?? '';
			const be = extOf(b.title ?? '') ?? '';
			return (EXT_PRIORITY[ae] ?? 999) - (EXT_PRIORITY[be] ?? 999);
		});
	});
	const hasIpfsFiles = $derived(playableIpfsFiles.length > 0);
	const firstIpfsCid = $derived(playableIpfsFiles[0]?.value ?? null);
	const hasMagnetFiles = $derived(firkin.files.some((f) => f.type === 'torrent magnet'));
	// "Real" files = anything playable on its own. URL-typed entries (TMDB
	// source URL, MusicBrainz release-group URL, persisted YouTube track
	// URLs) are pure metadata pointers and don't qualify.
	const hasNoRealFiles = $derived(!hasIpfsFiles && !hasMagnetFiles);
	const firkinKind = $derived(addonKind(firkin.addon));
	const isMusicBrainz = $derived(firkin.addon === 'musicbrainz');
	const isTmdbMovie = $derived(firkin.addon === 'tmdb-movie');
	const isTmdbTv = $derived(firkin.addon === 'tmdb-tv');
	const isYoutubeVideo = $derived(firkin.addon === 'youtube-video');

	function parseYouTubeWatchUrl(value: string): string | null {
		try {
			const u = new URL(value);
			const host = u.hostname.toLowerCase();
			if (host === 'youtu.be') return value;
			if (host === 'www.youtube.com' || host === 'youtube.com' || host === 'm.youtube.com') {
				if (u.pathname === '/watch' && u.searchParams.get('v')) return value;
			}
			return null;
		} catch {
			return null;
		}
	}

	const youtubeVideoUrl = $derived(
		firkin.files
			.map((f) => (f.type === 'url' ? parseYouTubeWatchUrl(f.value) : null))
			.find((u): u is string => Boolean(u)) ?? null
	);
	// Innertube client cached on a previous resolution (`web`, `web_embedded`,
	// `tv`, `android`, `ios`). Passed to `<CatalogTrailerPlayer>` so the
	// backend can skip the failing-candidate iteration on the happy path. A
	// stale value just falls back to the regular browser priority list.
	const youtubePreferredClient = $derived(
		firkin.files.find((f) => f.type === 'youtube preferred client')?.value ?? null
	);
	const thumb = $derived(firkin.images[0]?.url ?? null);
	// Trailers prefer the last image (typically the backdrop / wide art) so
	// the right-side player surfaces a 16:9 still rather than the poster.
	const trailerThumb = $derived(firkin.images[firkin.images.length - 1]?.url ?? thumb);

	const userIdentityState = userIdentityService.state;
	let recommendationsIngestedFor: string | null = null;

	function handleRelatedItemsLoaded(items: CatalogItem[]) {
		const sourceFirkinId = firkin.id;
		if (!sourceFirkinId) return;
		// Recommendation counts only update from real bookmarked detail
		// pages — non-bookmarked browse-cache firkins must behave like
		// the legacy /catalog/virtual page, which never ingested.
		if (!isBookmarked) return;
		if (recommendationsIngestedFor === sourceFirkinId) return;
		recommendationsIngestedFor = sourceFirkinId;
		const address = $userIdentityState.identity?.address;
		if (!address) return;
		if (items.length === 0) return;
		const ingestItems: RecommendationIngestItem[] = items
			.filter((it) => it.id && it.title)
			.map((it) => ({
				addon: firkin.addon,
				id: it.id,
				title: it.title,
				year: it.year,
				description: it.description,
				posterUrl: it.posterUrl,
				backdropUrl: it.backdropUrl,
				reviews: it.reviews ?? []
			}));
		void ingestRecommendations({ address, sourceFirkinId, items: ingestItems }).catch((err) => {
			console.warn('[recommendations] ingest failed:', err);
		});
	}

	const needsMetadata = $derived(firkin.description.trim() === '' || firkin.images.length === 0);
	const lookupAddon = $derived(metadataSearchAddon(firkin.addon));
	let metadataLookupOpen = $state(false);

	async function applyMetadata(item: CatalogLookupItem) {
		const updated = await firkinsService.enrich(firkin.id, {
			title: item.title,
			year: item.year,
			description: item.description ?? '',
			posterUrl: item.posterUrl,
			backdropUrl: item.backdropUrl
		});
		metadataLookupOpen = false;
		firkinOverride = updated;
	}

	function parseMusicBrainzReleaseGroupId(value: string): string | null {
		try {
			const u = new URL(value);
			if (u.hostname.toLowerCase() !== 'musicbrainz.org') return null;
			const m = u.pathname.match(/^\/release-group\/([^\/]+)/);
			return m?.[1] ?? null;
		} catch {
			return null;
		}
	}

	const musicBrainzReleaseGroupId = $derived(
		firkin.files
			.map((f) => (f.type === 'url' ? parseMusicBrainzReleaseGroupId(f.value) : null))
			.find((id): id is string => Boolean(id)) ?? null
	);

	function parseTmdbId(value: string, kind: 'tv' | 'movie'): string | null {
		try {
			const u = new URL(value);
			if (u.hostname.toLowerCase() !== 'www.themoviedb.org') return null;
			const re = new RegExp(`^/${kind}/([^/]+)`);
			const m = u.pathname.match(re);
			return m?.[1] ?? null;
		} catch {
			return null;
		}
	}

	const tmdbTvId = $derived(
		firkin.files
			.map((f) => (f.type === 'url' ? parseTmdbId(f.value, 'tv') : null))
			.find((id): id is string => Boolean(id)) ?? null
	);
	const tmdbMovieId = $derived(
		firkin.files
			.map((f) => (f.type === 'url' ? parseTmdbId(f.value, 'movie') : null))
			.find((id): id is string => Boolean(id)) ?? null
	);

	let ipfsStarting = $state(false);
	let ipfsError = $state<string | null>(null);

	type ArtistsBackfillStatus = 'idle' | 'loading' | 'done' | 'error';
	let artistsBackfillStatus = $state<ArtistsBackfillStatus>('idle');
	let artistsBackfillError = $state<string | null>(null);
	let metadataBackfillForFirkinId: string | null = null;

	function metadataUpstreamId(): string | null {
		if (isMusicBrainz) return musicBrainzReleaseGroupId;
		if (isTmdbMovie) return tmdbMovieId;
		if (isTmdbTv) return tmdbTvId;
		return null;
	}

	const OMDB_REVIEW_LABELS = new Set(['Rotten Tomatoes', 'Metacritic', 'IMDb']);

	$effect(() => {
		const fid = firkin.id;
		if (metadataBackfillForFirkinId === fid) return;
		const upstreamId = metadataUpstreamId();
		const existingReviews = firkin.reviews ?? [];
		const reviewsMissing = existingReviews.length === 0;
		// TMDB firkins created before OMDB_API_KEY was set (or before the key
		// was activated) only carry the TMDB review. Re-fetch metadata so the
		// server merges in Rotten Tomatoes / Metacritic / IMDb.
		const omdbApplies = isTmdbMovie || isTmdbTv;
		const omdbMissing =
			omdbApplies &&
			!existingReviews.some((r: { label: string }) => OMDB_REVIEW_LABELS.has(r.label));
		const fetchReviews = reviewsMissing || omdbMissing;
		const artistsMissing =
			firkin.artists.length === 0 && (isMusicBrainz || isTmdbMovie || isTmdbTv);
		if (!upstreamId || (!fetchReviews && !artistsMissing)) {
			metadataBackfillForFirkinId = fid;
			return;
		}
		metadataBackfillForFirkinId = fid;
		void backfillFromMetadata(fid, firkin.addon, upstreamId, {
			fetchArtists: artistsMissing,
			fetchReviews
		});
	});

	async function backfillFromMetadata(
		firkinId: string,
		addon: string,
		upstreamId: string,
		want: { fetchArtists: boolean; fetchReviews: boolean }
	) {
		if (want.fetchArtists) artistsBackfillStatus = 'loading';
		artistsBackfillError = null;
		try {
			const res = await fetch(
				`${base}/api/catalog/${encodeURIComponent(addon)}/${encodeURIComponent(upstreamId)}/metadata`,
				{ cache: 'no-store' }
			);
			if (!res.ok) throw new Error(`HTTP ${res.status}`);
			const body = (await res.json()) as {
				artists?: Firkin['artists'];
				reviews?: Firkin['reviews'];
			};
			const fetchedArtists = Array.isArray(body.artists) ? body.artists : [];
			const fetchedReviews = Array.isArray(body.reviews) ? body.reviews : [];
			const patch: { artists?: Firkin['artists']; reviews?: Firkin['reviews'] } = {};
			if (want.fetchArtists && fetchedArtists.length > 0) patch.artists = fetchedArtists;
			if (want.fetchReviews && fetchedReviews.length > 0) patch.reviews = fetchedReviews;
			if (Object.keys(patch).length === 0) {
				if (want.fetchArtists) artistsBackfillStatus = 'done';
				return;
			}
			const putRes = await fetch(`${base}/api/firkins/${encodeURIComponent(firkinId)}`, {
				method: 'PUT',
				headers: { 'content-type': 'application/json' },
				body: JSON.stringify(patch)
			});
			if (!putRes.ok) {
				let message = `HTTP ${putRes.status}`;
				try {
					const bb = await putRes.json();
					if (bb && typeof bb.error === 'string') message = bb.error;
				} catch {
					// ignore
				}
				throw new Error(message);
			}
			const updated = (await putRes.json()) as Firkin;
			firkinOverride = updated;
			if (want.fetchArtists) artistsBackfillStatus = 'done';
		} catch (err) {
			artistsBackfillError = err instanceof Error ? err.message : 'Unknown error';
			if (want.fetchArtists) artistsBackfillStatus = 'error';
		}
	}

	// Map "S{:02}E{:02}" → CID for every ipfs file whose title carries the
	// episode tag set by `tv_build.rs` (local-tv libraries) and
	// `torrent_completion.rs` (torrent downloads on tmdb-tv firkins).
	// Drives the seasons card's per-episode "Play" button.
	const episodeCidByKey = $derived.by(() => {
		const map = new Map<string, string>();
		for (const f of firkin.files) {
			if (f.type !== 'ipfs') continue;
			const m = (f.title ?? '').match(/^S(\d{1,3})E(\d{1,3})/i);
			if (!m) continue;
			const key = `S${m[1].padStart(2, '0')}E${m[2].padStart(2, '0')}`;
			if (!map.has(key)) map.set(key, f.value);
		}
		return map;
	});
	const availableEpisodeKeys = $derived(new Set(episodeCidByKey.keys()));

	// Active IPFS-stream session. We retain the session id so we can
	// `DELETE` it before starting a new one (e.g. on a seek-anywhere
	// click that lands outside the buffered range), and the source CID
	// + label so the seek handler knows what stream to restart.
	let ipfsSessionId = $state<string | null>(null);
	let ipfsSessionCid = $state<string | null>(null);
	let ipfsSessionLabel = $state<string | null>(null);

	async function playIpfsCid(
		cid: string,
		label: string,
		seekPositionSecs: number = 0
	): Promise<void> {
		if (!cid || ipfsStarting) return;
		ipfsStarting = true;
		ipfsError = null;
		// Tear down the previous session before kicking off a new one so
		// the cloud isn't transcoding two streams of the same source in
		// parallel after a seek-anywhere restart.
		const prevSessionId = ipfsSessionId;
		if (prevSessionId) {
			void fetch(`${base}/api/ipfs-stream/sessions/${encodeURIComponent(prevSessionId)}`, {
				method: 'DELETE'
			}).catch(() => {});
			ipfsSessionId = null;
		}
		try {
			const res = await fetch(`${base}/api/ipfs-stream/sessions`, {
				method: 'POST',
				headers: { 'content-type': 'application/json' },
				body: JSON.stringify({
					cid,
					seekPositionSeconds: seekPositionSecs > 0 ? seekPositionSecs : undefined
				})
			});
			if (!res.ok) {
				let message = `HTTP ${res.status}`;
				try {
					const body = await res.json();
					if (body && typeof body.error === 'string') message = body.error;
				} catch {
					// ignore
				}
				throw new Error(message);
			}
			const body = (await res.json()) as {
				sessionId: string;
				playlistUrl: string;
				durationSeconds?: number;
			};
			const durationSecs =
				typeof body.durationSeconds === 'number' && body.durationSeconds > 0
					? body.durationSeconds
					: null;
			const file: PlayableFile = {
				id: `firkin:${firkin.id}:ipfs:${cid}`,
				type: 'library',
				name: label,
				outputPath: '',
				mode: 'video',
				format: null,
				videoFormat: null,
				thumbnailUrl: thumb,
				durationSeconds: durationSecs,
				size: 0,
				completedAt: '',
				subtitles: playerSubtitles
			};
			ipfsSessionId = body.sessionId;
			ipfsSessionCid = cid;
			ipfsSessionLabel = label;
			await playerService.playUrl(
				file,
				body.playlistUrl,
				'application/vnd.apple.mpegurl',
				'inline',
				firkin.id,
				null,
				null,
				null,
				seekPositionSecs > 0 ? seekPositionSecs : 0
			);
		} catch (err) {
			ipfsError = err instanceof Error ? err.message : 'Unknown error';
			ipfsSessionId = null;
			// Pipeline never produced a playlist — go back to the trailer tab
			// so the user isn't stuck on a permanent spinner. The error alert
			// above the player still surfaces what went wrong.
			if (activeSource === 'ipfs') activeSource = 'trailer';
		} finally {
			ipfsStarting = false;
		}
	}

	// Out-of-buffer seek handler for the inline player. PlayerVideo
	// already does in-element seeks for targets that fall inside the
	// buffered range; only the past-end / before-offset cases reach
	// here, and the only correct response is "transcode from there".
	async function handleIpfsSeek(targetSourceSecs: number): Promise<void> {
		const cid = ipfsSessionCid;
		const label = ipfsSessionLabel;
		if (!cid || !label) return;
		const target = Math.max(0, Math.floor(targetSourceSecs * 1000) / 1000);
		await playIpfsCid(cid, label, target);
	}

	async function startIpfsPlay(): Promise<void> {
		if (!firstIpfsCid) return;
		await playIpfsCid(firstIpfsCid, firkin.title);
	}

	// Track which TV episode the user is currently streaming so the
	// subtitle search and the player's filtered subtitle list can pivot
	// per-episode. Set in `playEpisode`; left null for movies and any
	// non-episode IPFS / torrent stream.
	let currentSeason = $state<number | null>(null);
	let currentEpisode = $state<number | null>(null);

	async function playEpisode(season: number, episode: number): Promise<void> {
		const key = `S${String(season).padStart(2, '0')}E${String(episode).padStart(2, '0')}`;
		const cid = episodeCidByKey.get(key);
		if (!cid) return;
		// Switch the player tab so the inline pane renders the episode.
		// The IPFS tab is gated on `ipfsTabEnabled` (= `hasIpfsFiles`),
		// which is true whenever any episode has a CID.
		currentSeason = season;
		currentEpisode = episode;
		activeSource = 'ipfs';
		await playIpfsCid(cid, `${firkin.title} — ${key}`);
	}

	const torrentsState = firkinTorrentsService.state;
	onMount(() => firkinTorrentsService.start());

	const firstMagnet = $derived(
		firkin.files.find((f) => f.type === 'torrent magnet')?.value ?? null
	);
	// Persisted stream pick from the attachment card. Present when the user
	// has previously kicked off a torrent-stream — without a download
	// magnet attached — via the search row's Stream button.
	const firstStreamMagnet = $derived(
		firkin.files.find((f) => f.type === 'torrent stream magnet')?.value ?? null
	);

	let torrentStreamStarting = $state(false);
	let torrentStreamError = $state<string | null>(null);

	const trailerTabEnabled = $derived(
		isTmdbMovie || isTmdbTv || (isYoutubeVideo && Boolean(youtubeVideoUrl))
	);
	const ipfsTabEnabled = $derived(hasIpfsFiles);
	// Once the file is pinned locally to IPFS, the torrent stream is
	// strictly worse — same bytes, slower path, extra peers — so we
	// hide that option entirely.
	const torrentTabEnabled = $derived(
		(Boolean(firstMagnet) || Boolean(firstStreamMagnet)) && !ipfsTabEnabled
	);
	const anyTabEnabled = $derived(trailerTabEnabled || ipfsTabEnabled || torrentTabEnabled);

	const trailerTabTitle = $derived(trailerTabEnabled ? 'Show trailer' : 'No trailer for this item');
	const ipfsTabTitle = $derived(
		ipfsTabEnabled
			? 'Stream over IPFS as HLS'
			: 'Available once at least one file is pinned to IPFS'
	);
	const torrentTabTitle = $derived.by(() => {
		if (ipfsTabEnabled) return 'File is pinned locally — use IPFS Stream instead';
		if (!firstMagnet && !firstStreamMagnet) return 'Available once a torrent magnet is attached';
		return 'Stream the attached torrent as it downloads';
	});
	const torrentTabSuffix = $derived(torrentStreamStarting ? ' — starting…' : '');

	// If IPFS becomes available while the user is on the torrent tab,
	// flip them over — the torrent tab is now disabled, so leaving them
	// on it would strand them on a stale player.
	$effect(() => {
		if (ipfsTabEnabled && activeSource === 'torrent') {
			activeSource = 'ipfs';
		}
	});

	function selectSource(source: StreamSource): void {
		if (source === activeSource) return;
		activeSource = source;
		if (source === 'trailer') {
			const s = get(playerService.state);
			const m = get(playerService.displayMode);
			if (m === 'inline' && s.firkinId === firkin.id && Boolean(s.directStreamUrl)) {
				void playerService.stop();
			}
		} else if (source === 'ipfs') {
			void startIpfsPlay();
		} else if (source === 'torrent') {
			void startTorrentStream();
		}
	}

	function handleSourceClick(source: StreamSource): void {
		if (source === 'trailer') {
			if (activeSource !== 'trailer') selectSource('trailer');
			return;
		}
		if (source === activeSource) {
			if (source === 'ipfs') void startIpfsPlay();
			else if (source === 'torrent') void startTorrentStream();
			return;
		}
		selectSource(source);
	}

	let streamingHash = $state<string | null>(null);

	async function startTorrentStream(magnetOverride?: string): Promise<boolean> {
		// Prefer the explicit stream pick (set by the attachment card / search
		// row) over the download magnet, then fall back to the download magnet
		// when no stream pick has been made yet.
		const magnet = magnetOverride ?? firstStreamMagnet ?? firstMagnet;
		if (!magnet || torrentStreamStarting) return false;
		torrentStreamStarting = true;
		streamingHash = magnet;
		torrentStreamError = null;
		try {
			const res = await fetch(`${base}/api/torrent/stream`, {
				method: 'POST',
				headers: { 'content-type': 'application/json' },
				body: JSON.stringify({ magnet })
			});
			if (!res.ok) {
				let message = `HTTP ${res.status}`;
				try {
					const body = await res.json();
					if (body && typeof body.error === 'string') message = body.error;
				} catch {
					// ignore
				}
				throw new Error(message);
			}
			const body = (await res.json()) as {
				infoHash: string;
				name: string;
				fileIndex: number;
				fileName: string;
				fileSize: number;
				mimeType: string | null;
				streamUrl: string;
			};
			const file: PlayableFile = {
				id: `firkin:${firkin.id}:torrent:${body.infoHash}:${body.fileIndex}`,
				type: 'library',
				name: body.fileName || firkin.title,
				outputPath: '',
				mode: 'video',
				format: null,
				videoFormat: null,
				thumbnailUrl: thumb,
				durationSeconds: null,
				size: body.fileSize,
				completedAt: '',
				subtitles: playerSubtitles
			};
			await playerService.playUrl(file, body.streamUrl, body.mimeType ?? null, 'inline', firkin.id);
			return true;
		} catch (err) {
			torrentStreamError = err instanceof Error ? err.message : 'Unknown error';
			if (activeSource === 'torrent') activeSource = 'trailer';
			return false;
		} finally {
			torrentStreamStarting = false;
			streamingHash = null;
		}
	}

	/// Start a torrent stream for a magnet, switch the player to the
	/// inline torrent source, and persist the user's pick on the firkin so
	/// the attachment card remembers it across reloads. Re-playing the same
	/// magnet (e.g. via the attachment card's stream cell) skips the
	/// persist round-trip — the firkin already records this exact magnet.
	///
	/// `start_stream` writes to the dedicated `downloads/torrent-streams/`
	/// directory and wipes any prior stream payload, so the persisted
	/// magnet always matches the torrent currently in that directory.
	async function playStreamFor(magnet: string, title: string | null): Promise<void> {
		if (!magnet || torrentStreamStarting) return;
		activeSource = 'torrent';
		const ok = await startTorrentStream(magnet);
		if (!ok) return;
		const alreadyPersisted = firkin.files.some(
			(f) => f.type === 'torrent stream magnet' && f.value === magnet
		);
		if (alreadyPersisted) return;
		try {
			const updated = await firkinsService.mutateFiles(firkin.id, {
				removeTypes: ['torrent stream magnet'],
				add: [{ type: 'torrent stream magnet', value: magnet, title: title ?? undefined }]
			});
			firkinOverride = updated;
		} catch (err) {
			console.warn('[catalog detail] failed to persist stream magnet:', err);
		}
	}

	/// Search-row Stream button — pick a brand-new torrent for streaming.
	async function streamTorrentFromRow(torrent: TorrentResultItem): Promise<void> {
		if (!torrent.magnetLink) return;
		await ensureBookmarked();
		await playStreamFor(torrent.magnetLink, torrent.title);
	}

	/// Attachment-card Stream cell click — re-run the torrent-stream stack
	/// on whatever's already persisted on the firkin. Useful after a page
	/// reload (the on-disk stream payload may have been wiped by another
	/// firkin's pick) or for replaying the current pick after the player
	/// was stopped.
	async function replayStreamMagnet(): Promise<void> {
		const entry = firkin.files.find((f) => f.type === 'torrent stream magnet' && f.value);
		if (!entry || !entry.value) return;
		await playStreamFor(entry.value, entry.title ?? null);
	}

	/// Attachment-card Download cell click — only fires once
	/// `torrent_completion.rs` has pinned the finished torrent to IPFS, at
	/// which point the cell switches into IPFS-play mode. Flips the player
	/// tab to `ipfs` and runs the existing IPFS HLS pipeline on the first
	/// playable CID (same path as the IPFS Stream tab).
	async function playFromAttachmentDownload(): Promise<void> {
		if (!firstIpfsCid || ipfsStarting) return;
		activeSource = 'ipfs';
		await startIpfsPlay();
	}

	const completedTorrents = $derived.by(() => {
		const out: { hash: string; title: string }[] = [];
		for (const f of firkin.files) {
			if (f.type !== 'torrent magnet' || !f.value) continue;
			const hash = infoHashFromMagnet(f.value);
			if (!hash) continue;
			const t = $torrentsState.byHash[hash];
			if (!t) continue;
			const finished = t.state === 'seeding' || t.progress >= 1;
			if (finished) out.push({ hash, title: f.title ?? t.name });
		}
		return out;
	});

	const canPlay = $derived(hasIpfsFiles || completedTorrents.length > 0);

	let finalizing = $state(false);
	let finalizeError = $state<string | null>(null);

	async function play() {
		if (hasIpfsFiles) {
			firkinPlaybackService.select(firkin as CloudFirkin);
			return;
		}
		if (finalizing) return;
		finalizeError = null;
		finalizing = true;
		try {
			const res = await fetch(`${base}/api/firkins/${encodeURIComponent(firkin.id)}/finalize`, {
				method: 'POST'
			});
			if (!res.ok) {
				let message = `HTTP ${res.status}`;
				try {
					const body = await res.json();
					if (body && typeof body.error === 'string') message = body.error;
				} catch {
					// ignore
				}
				throw new Error(message);
			}
			const next = (await res.json()) as Firkin;
			firkinOverride = next;
			if (next.files.some((f) => f.type === 'ipfs')) {
				firkinPlaybackService.select(next as unknown as CloudFirkin);
			}
		} catch (err) {
			finalizeError = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			finalizing = false;
		}
	}

	// Detail mode persists trailer/track URLs back to the firkin so they
	// don't have to be re-resolved every visit. The PUT updates the
	// record in place — the firkin's UUID id is stable across versions —
	// and the response carries the recomputed `cid`.
	async function persistFirkinPatch(patch: Partial<Firkin>): Promise<void> {
		const res = await fetch(`${base}/api/firkins/${encodeURIComponent(firkin.id)}`, {
			method: 'PUT',
			headers: { 'content-type': 'application/json' },
			body: JSON.stringify(patch)
		});
		if (!res.ok) {
			let message = `HTTP ${res.status}`;
			try {
				const body = await res.json();
				if (body && typeof body.error === 'string') message = body.error;
			} catch {
				// ignore
			}
			throw new Error(message);
		}
		const updated = (await res.json()) as Firkin;
		firkinOverride = updated;
	}

	// Cache the Innertube client that produced a successful trailer / video
	// resolution on this firkin. Stored as a single `youtube preferred
	// client` file row; rolls the firkin version forward only when the
	// value is new or has changed (skipping the rollforward when the cache
	// is already accurate).
	async function persistYoutubePreferredClient(clientName: string): Promise<void> {
		if (!clientName) return;
		// Skip the rollforward on browse-cache firkins; persisting would
		// roll the CID forward on a record the user hasn't committed to.
		if (!isBookmarked) return;
		if (youtubePreferredClient === clientName) return;
		// Atomic: server reads current `files` under the per-firkin lock,
		// drops any prior `youtube preferred client` entry, appends the
		// new one, and rolls forward. This is the canonical fix for the
		// "torrent magnet vanished from the firkin" race — the legacy
		// PUT path used to clobber a freshly-attached magnet whenever
		// this client-cache write landed in the same window.
		try {
			const updated = await firkinsService.mutateFiles(firkin.id, {
				removeTypes: ['youtube preferred client'],
				add: [{ type: 'youtube preferred client', value: clientName }]
			});
			firkinOverride = updated;
		} catch (err) {
			console.warn('[catalog detail] persist youtube preferred client failed:', err);
		}
	}

	// Persist trailer / track resolutions back to the firkin only while
	// bookmarked — browse-cache firkins behave like the legacy
	// `/catalog/virtual` page (resolve for display, never roll the CID).
	const trailerResolver = new TrailerResolver({
		persist: (resolved) =>
			isBookmarked ? persistFirkinPatch({ trailers: resolved }) : Promise.resolve()
	});
	// All playable trailers — feeds the top-right select on the inline
	// `CatalogTrailerPlayer` so the user can switch between movie + per-season
	// trailers without a separate trailers list.
	const playableTrailers = $derived(
		trailerResolver.trailers
			.filter((t): t is typeof t & { youtubeUrl: string } => Boolean(t.youtubeUrl))
			.map((t) => ({ key: t.key, label: t.label, youtubeUrl: t.youtubeUrl }))
	);
	let selectedTrailerKey = $state<string | null>(null);
	$effect(() => {
		const keys = playableTrailers.map((t) => t.key);
		if (keys.length === 0) {
			selectedTrailerKey = null;
			return;
		}
		if (!selectedTrailerKey || !keys.includes(selectedTrailerKey)) {
			selectedTrailerKey = keys[0];
		}
	});
	const firstTrailerUrl = $derived(
		playableTrailers.find((t) => t.key === selectedTrailerKey)?.youtubeUrl ??
			playableTrailers[0]?.youtubeUrl ??
			null
	);
	let trailersInitForFirkinId: string | null = null;

	$effect(() => {
		if (!isTmdbMovie && !isTmdbTv) return;
		const fid = firkin.id;
		if (trailersInitForFirkinId === fid) return;
		trailersInitForFirkinId = fid;
		const stored = firkin.trailers ?? [];
		if (isTmdbMovie) {
			void trailerResolver.resolveMovie({
				addon: firkin.addon,
				tmdbMovieId,
				title: firkin.title,
				year: firkin.year,
				stored
			});
		} else {
			void trailerResolver.resolveTv({
				addon: firkin.addon,
				tmdbTvId,
				title: firkin.title,
				stored
			});
		}
	});

	const trackResolver = new TrackResolver();
	let tracksInitForFirkinId: string | null = null;

	$effect(() => {
		if (!isMusicBrainz) return;
		const fid = firkin.id;
		if (tracksInitForFirkinId === fid) return;
		tracksInitForFirkinId = fid;
		if (musicBrainzReleaseGroupId) {
			void trackResolver.loadFromFirkin({
				releaseGroupId: musicBrainzReleaseGroupId,
				files: firkin.files
			});
		}
	});

	// Heuristic the polling effect uses to decide whether the server's
	// background album-resolution task is still running. Any musicbrainz
	// firkin with a release-group url but no `lyrics`-typed file entries
	// is treated as "not yet resolved" and polled. Keying off
	// `firkin.files` directly (rather than `loadFromFirkin`'s missingAny)
	// makes polling robust to MusicBrainz being slow / rate-limited —
	// even if the WebUI can't render the tracklist, the server is still
	// processing and we still want to navigate to the rollforward.
	const tracksLikelyUnresolved = $derived(
		isMusicBrainz &&
			Boolean(musicBrainzReleaseGroupId) &&
			firkin.files.filter((f) => f.type === 'lyrics').length === 0
	);

	// While the server's background album-resolution task is still
	// running, poll the firkin every few seconds. The record id (UUID)
	// is stable across version updates; we just need to refresh the body
	// so the resolved track URLs / lyrics show up.
	$effect(() => {
		if (!tracksLikelyUnresolved) return;
		const id = firkin.id;
		const releaseGroupId = musicBrainzReleaseGroupId;
		let cancelled = false;
		const tick = async () => {
			if (cancelled) return;
			try {
				const res = await fetch(`${base}/api/firkins/${encodeURIComponent(id)}`, {
					cache: 'no-store'
				});
				if (cancelled) return;
				if (!res.ok) return;
				const fresh = (await res.json()) as Firkin;
				if (cancelled) return;
				const freshHasMore =
					fresh.files.length !== firkin.files.length || fresh.updated_at !== firkin.updated_at;
				if (freshHasMore) {
					firkinOverride = fresh;
					if (releaseGroupId) {
						void trackResolver.loadFromFirkin({
							releaseGroupId,
							files: fresh.files
						});
					}
				}
			} catch {
				// swallow — try again on next tick
			}
		};
		const timer = setInterval(tick, 4000);
		return () => {
			cancelled = true;
			clearInterval(timer);
		};
	});

	// Real-time per-track resolution progress. Independent of the
	// firkin-rollforward poll above: this polls the in-memory progress
	// map every second so each track's YT URL / lyrics status flips
	// from `pending` → `searching` → `found` / `missing` as the server
	// task works through the tracklist. Stops once `completed` flips
	// true (the firkin poll handles the navigation to the rolled-forward
	// id).
	$effect(() => {
		if (!tracksLikelyUnresolved) return;
		const id = firkin.id;
		let cancelled = false;
		let stopped = false;
		const tick = async () => {
			if (cancelled || stopped) return;
			try {
				const res = await fetch(
					`${base}/api/firkins/${encodeURIComponent(id)}/resolution-progress`,
					{ cache: 'no-store' }
				);
				if (cancelled) return;
				if (!res.ok) return;
				const payload = (await res.json()) as AlbumProgressPayload;
				trackResolver.applyProgress(payload);
				if (payload.completed) {
					stopped = true;
				}
			} catch {
				// swallow — try again on next tick
			}
		};
		void tick();
		const timer = setInterval(tick, 1000);
		return () => {
			cancelled = true;
			clearInterval(timer);
		};
	});

	// Album download — kicked by the tracks card's "Download album" button.
	// `POST /api/firkins/:id/download-album` spawns a server-side task that
	// walks the tracklist, downloads each track's persisted YouTube URL via
	// yt-dlp (audio-only), pins the file to IPFS, and rolls the firkin
	// forward. The browser only triggers and observes — the task survives
	// page reloads and tab closures.
	let albumDownloadInFlight = $state(false);
	let albumDownloadError = $state<string | null>(null);

	async function startAlbumDownload(): Promise<void> {
		if (albumDownloadInFlight) return;
		albumDownloadInFlight = true;
		albumDownloadError = null;
		try {
			const res = await fetch(
				`${base}/api/firkins/${encodeURIComponent(firkin.id)}/download-album`,
				{ method: 'POST' }
			);
			if (!res.ok) {
				let message = `HTTP ${res.status}`;
				try {
					const body = await res.json();
					if (body && typeof body.error === 'string') message = body.error;
				} catch {
					// ignore
				}
				throw new Error(message);
			}
		} catch (err) {
			albumDownloadError = err instanceof Error ? err.message : 'Unknown error';
			albumDownloadInFlight = false;
		}
	}

	// Re-hydrate album-download state when a task is already running
	// server-side (e.g. the user reloaded the page mid-download). The
	// progress endpoint returns 404 when nothing is in flight; any 200
	// with `completed: false` flips the in-flight flag back on so the
	// poll below picks up where it left off.
	$effect(() => {
		if (!isMusicBrainz) return;
		const id = firkin.id;
		let cancelled = false;
		void (async () => {
			try {
				const res = await fetch(`${base}/api/firkins/${encodeURIComponent(id)}/download-progress`, {
					cache: 'no-store'
				});
				if (cancelled || !res.ok) return;
				const payload = (await res.json()) as AlbumDownloadProgressPayload;
				trackResolver.applyDownloadProgress(payload);
				if (!payload.completed) albumDownloadInFlight = true;
				else if (payload.error) albumDownloadError = payload.error;
			} catch {
				// no in-flight task — ignore
			}
		})();
		return () => {
			cancelled = true;
		};
	});

	// Poll album download progress while the task is running, projecting
	// per-track status onto the existing tracks via `applyDownloadProgress`.
	// The firkin poll above picks up rolled-forward `ipfs` file entries and
	// re-projects them via `loadFromFirkin`, so the per-row "Play" button
	// surfaces as soon as a track finishes — without waiting for the
	// entire album to complete.
	$effect(() => {
		if (!isMusicBrainz) return;
		if (!albumDownloadInFlight) return;
		const id = firkin.id;
		let cancelled = false;
		const tick = async () => {
			if (cancelled) return;
			try {
				const res = await fetch(`${base}/api/firkins/${encodeURIComponent(id)}/download-progress`, {
					cache: 'no-store'
				});
				if (cancelled) return;
				if (!res.ok) return;
				const payload = (await res.json()) as AlbumDownloadProgressPayload;
				trackResolver.applyDownloadProgress(payload);
				if (payload.completed) {
					albumDownloadInFlight = false;
					if (payload.error) albumDownloadError = payload.error;
				}
			} catch {
				// swallow — try again on next tick
			}
		};
		void tick();
		const timer = setInterval(tick, 1500);
		return () => {
			cancelled = true;
			clearInterval(timer);
		};
	});

	// Per-row streamability probes drive the Stream button's enabled state in
	// `CatalogTorrentSearchCard`. With the streamability whitelist now narrowed
	// to mp4/m4v/webm (the only containers a Tauri WebView <video> element can
	// actually progressive-stream), the eval verdict is meaningful: rows that
	// come back `not-streamable` get the button disabled instead of letting
	// the user click and get hit with "no streamable video file in torrent"
	// from the Stream endpoint. Probes are cached server-side in `torrent_eval`
	// keyed by info_hash, so repeated visits are instant.
	const torrentSearch = new TorrentSearch({ evaluate: true });

	// Season-aware fan-out for tv shows. After the initial show-name search
	// settles, we look at how its results classify by season — any season
	// the show has but the results don't cover gets its own focused search
	// (`Show Name S05`). Untagged results (no `S0N` token) are not treated
	// as whole-show packs: most are episode rips or release-group dumps with
	// sloppy naming, not complete-series torrents, so they don't suppress
	// the per-season fan-out. searchAppend dedups by infoHash, so re-finding
	// a torrent already in the show-wide pool is harmless.
	let tvSeasonNumbers = $state<number[]>([]);
	let perSeasonSearched: { firkinId: string; seasons: Set<number> } | null = null;
	const tvResultCoverage = $derived.by<{ covered: Set<number> }>(() => {
		const covered = new Set<number>();
		for (const t of torrentSearch.matches) {
			const range = parseTorrentSeasons(t.parsedTitle || t.title);
			if (!range) continue;
			for (let s = range.start; s <= range.end; s++) covered.add(s);
		}
		return { covered };
	});
	let addingHash = $state<string | null>(null);
	let assignError = $state<string | null>(null);
	let startedHashes = $state<Set<string>>(new Set());
	let torrentSearchOpen = $state(false);
	let torrentSearchInitForFirkinId: string | null = null;
	let subsLyricsOpen = $state(false);
	let filesOpen = $state(false);

	const existingHashes = $derived(
		new Set(firkin.files.filter((f) => f.type === 'torrent magnet' && f.value).map((f) => f.value))
	);

	/// Look up an attached magnet's metadata in the persisted torrent search
	/// results so the attachment card can show seeders/leechers/size pulled
	/// from the indexer's snapshot. Falls back to the FileEntry's own title
	/// when the search cache doesn't carry the picked infoHash (e.g. the
	/// user picked it before the indexer's results were persisted, or a
	/// later refresh dropped it).
	function attachmentInfoFor(
		fileType: 'torrent magnet' | 'torrent stream magnet'
	): AttachmentInfo | null {
		const entry = firkin.files.find((f) => f.type === fileType && f.value);
		if (!entry || !entry.value) return null;
		const hash = infoHashFromMagnet(entry.value);
		const match = hash
			? torrentSearch.matches.find((m) => m.infoHash.toLowerCase() === hash.toLowerCase())
			: null;
		return {
			title: match?.parsedTitle || match?.title || entry.title || 'Magnet attached',
			seeders: match?.seeders ?? null,
			leechers: match?.leechers ?? null,
			sizeBytes: match?.sizeBytes ?? null
		};
	}

	/// Same lookup as `attachmentInfoFor('torrent magnet')` but joined with
	/// the live `firkinTorrentsService` snapshot so the attachment card can
	/// render actual download progress / speed / ETA. Live `size` (resolved
	/// post-metadata) wins over the indexer's static `sizeBytes`.
	const downloadInfo = $derived.by<DownloadAttachmentInfo | null>(() => {
		const base = attachmentInfoFor('torrent magnet');
		if (!base) return null;
		const entry = firkin.files.find((f) => f.type === 'torrent magnet' && f.value);
		const hash = entry?.value ? infoHashFromMagnet(entry.value) : null;
		const live = hash ? $torrentsState.byHash[hash] : null;
		// Once `torrent_completion.rs` pins the finished torrent's files to
		// IPFS, an `ipfs`-typed FileEntry shows up on the firkin and the
		// cell flips into a click-to-stream state. We use `firstIpfsCid`
		// (filtered to playable types) so the click is guaranteed to land
		// on something the player can render.
		const ipfsCid = firstIpfsCid;
		if (!live) {
			return {
				...base,
				progress: null,
				downloadSpeed: null,
				etaSeconds: null,
				finished: false,
				ipfsCid
			};
		}
		return {
			...base,
			sizeBytes: live.size > 0 ? live.size : base.sizeBytes,
			progress: live.progress,
			downloadSpeed: live.downloadSpeed,
			etaSeconds: live.eta,
			finished: live.state === 'seeding' || live.progress >= 1,
			ipfsCid
		};
	});
	const streamInfo = $derived(attachmentInfoFor('torrent stream magnet'));

	// Surface a faded "preferred pick" preview in each empty attachment cell.
	// Walks `torrentSearch.groupedMatches` top-down — groups are already in
	// quality priority order (4K → 2160p → 1080p → 720p → 480p → 360p → Other)
	// and rows within a group are sorted by seeders desc — so the first row
	// that matches the per-cell criteria is the highest-quality / most-seeded
	// option whose corresponding button would be enabled in the search table.
	const preferredDownloadTorrent = $derived.by<TorrentResultItem | null>(() => {
		for (const group of torrentSearch.groupedMatches) {
			for (const row of group.rows) {
				if (!row.magnetLink) continue;
				if (existingHashes.has(row.magnetLink)) continue;
				return row;
			}
		}
		return null;
	});
	const preferredStreamTorrent = $derived.by<TorrentResultItem | null>(() => {
		for (const group of torrentSearch.groupedMatches) {
			for (const row of group.rows) {
				if (!row.magnetLink) continue;
				if (torrentSearch.rowEvals[row.magnetLink]?.kind !== 'streamable') continue;
				return row;
			}
		}
		return null;
	});

	function toggleTorrentSearch() {
		torrentSearchOpen = !torrentSearchOpen;
		if (torrentSearchOpen && torrentSearchInitForFirkinId !== firkin.id) {
			torrentSearchInitForFirkinId = firkin.id;
			void torrentSearch.search({
				addon: firkin.addon,
				title: firkin.title,
				year: firkin.year,
				firkinId: firkin.id
			});
		}
	}

	// When the firkin is just metadata (no IPFS files, no magnets), kick
	// the torrent search off automatically so the user can pick a source
	// without having to click into a collapsed card. MusicBrainz is
	// excluded because albums get the tracks card, not torrents.
	//
	// TV shows ignore the `hasNoRealFiles` gate: the seasons card needs the
	// search results to classify torrents per-season, and the user typically
	// adds one torrent per season — so even when the firkin already has a
	// magnet attached, fresh results are still useful.
	//
	// Movies also ignore the gate when either attachment cell is empty: the
	// `CatalogTorrentAttachmentCard` renders a faded preferred-pick preview
	// in the empty cell, and the preview needs `torrentSearch.matches` to
	// have populated regardless of whether the *other* cell is already
	// filled.
	$effect(() => {
		if (isMusicBrainz) return;
		const movieAttachmentEmpty =
			isTmdbMovie && (downloadInfo === null || streamInfo === null);
		if (!isTmdbTv && !hasNoRealFiles && !movieAttachmentEmpty) return;
		if (torrentSearchInitForFirkinId === firkin.id) return;
		torrentSearchInitForFirkinId = firkin.id;
		void torrentSearch.search({
			addon: firkin.addon,
			title: firkin.title,
			year: firkin.year,
			firkinId: firkin.id
		});
	});

	// Once the initial show-name search settles, classify its results by
	// season and fan out a focused `Show Title S0N` search for every season
	// that has zero matches in the initial pool. Skipped entirely if a
	// whole-show pack already appeared in results — that covers everything.
	// Per-season fires are guarded by `perSeasonSearched` so each season
	// only ever runs once per firkin id.
	$effect(() => {
		if (!isTmdbTv) return;
		if (tvSeasonNumbers.length === 0) return;
		if (torrentSearch.status !== 'done') return;
		const { covered } = tvResultCoverage;
		if (perSeasonSearched?.firkinId !== firkin.id) {
			perSeasonSearched = { firkinId: firkin.id, seasons: new Set() };
		}
		const searched = perSeasonSearched.seasons;
		for (const n of tvSeasonNumbers) {
			if (n <= 0) continue;
			if (covered.has(n)) continue;
			if (searched.has(n)) continue;
			searched.add(n);
			const tag = `S${String(n).padStart(2, '0')}`;
			void torrentSearch.searchAppend(
				firkin.addon,
				`${firkin.title} ${tag}`,
				firkin.title,
				firkin.year,
				`Season ${n}`
			);
		}
	});

	const subsLyricsResolver = new SubsLyricsResolver();
	// Subtitle search runs for both TMDB movies and TMDB TV shows. TV
	// requires the user to have started an episode-specific stream first
	// (so we know which `(season, episode)` to pin the OpenSubtitles
	// query to); the picker UI gates on `currentSeason`/`currentEpisode`
	// being set.
	const subsLyricsKind = $derived<'subs' | null>(isTmdbMovie || isTmdbTv ? 'subs' : null);
	// Per-(firkin, season, episode) sentinel — for TV we want the search
	// to re-fire whenever the user pivots to a different episode, so a
	// flat firkin-id sentinel won't do.
	let subsLyricsInitKey: string | null = null;

	const subsLyricsSearchTerm = $derived.by<string | null>(() => {
		if (!subsLyricsKind) return null;
		if (isTmdbTv) {
			if (!tmdbTvId) {
				return `no TMDB id on this firkin (title: ${firkin.title})`;
			}
			if (currentSeason === null || currentEpisode === null) {
				return `${firkin.title} — start an episode stream to search subtitles for it`;
			}
			const tag = `S${String(currentSeason).padStart(2, '0')}E${String(currentEpisode).padStart(2, '0')}`;
			return `OpenSubtitles v3 tv via TMDB id=${tmdbTvId} ${tag} (title: ${firkin.title})`;
		}
		if (!tmdbMovieId) {
			return `no TMDB id on this firkin (title: ${firkin.title})`;
		}
		return `OpenSubtitles v3 movie via TMDB id=${tmdbMovieId} (title: ${firkin.title})`;
	});

	function runSubsLyricsSearch() {
		if (!subsLyricsKind) return;
		if (isTmdbTv) {
			if (!tmdbTvId || currentSeason === null || currentEpisode === null) return;
			void subsLyricsResolver.search({
				addon: firkin.addon,
				query: firkin.title,
				externalIds: [tmdbTvId],
				season: currentSeason,
				episode: currentEpisode
			});
			return;
		}
		if (!tmdbMovieId) return;
		void subsLyricsResolver.search({
			addon: firkin.addon,
			query: firkin.title,
			externalIds: [tmdbMovieId]
		});
	}

	// Subtitle search no longer auto-fires on detail mount — it kicks
	// only once an IPFS or torrent stream actually starts. For TV the
	// search also re-fires when the user pivots to a different episode,
	// so the gate is keyed off `(firkin.id, currentSeason, currentEpisode)`
	// rather than the firkin id alone.
	$effect(() => {
		if (!subsLyricsKind) return;
		if (activeSource !== 'ipfs' && activeSource !== 'torrent') return;
		let key: string;
		if (isTmdbTv) {
			if (!tmdbTvId) return;
			if (currentSeason === null || currentEpisode === null) return;
			key = `${firkin.id}::${currentSeason}::${currentEpisode}`;
		} else {
			if (!tmdbMovieId) return;
			key = firkin.id;
		}
		if (subsLyricsInitKey === key) return;
		subsLyricsInitKey = key;
		// Drop any prior episode-specific selection so the new search
		// doesn't render last episode's pick as `selected`.
		selectedSubExternalId = null;
		runSubsLyricsSearch();
	});

	// Subtitles already attached to the firkin (downloaded + pinned by a
	// previous `POST /api/firkins/:id/subtitle` round). Each row is a
	// JSON-encoded `SubtitleFileValue` whose `cid` points at the VTT
	// bytes — we resolve through `/api/ipfs/pins/<cid>/file` so the
	// player can stream it directly. Anything that fails to parse is
	// dropped silently (legacy `assignTorrent` payloads use a different
	// shape and don't carry a cid yet).
	type AttachedSubtitle = {
		cid: string;
		language: string;
		display: string;
		release: string | null;
		externalId: string;
		season: number | null;
		episode: number | null;
	};
	const attachedSubtitles = $derived.by<AttachedSubtitle[]>(() => {
		const out: AttachedSubtitle[] = [];
		for (const f of firkin.files) {
			if (f.type !== 'subtitle' || !f.value) continue;
			try {
				const parsed = JSON.parse(f.value) as Partial<{
					cid: string;
					language: string;
					display: string;
					release: string | null;
					externalId: string;
					season: number;
					episode: number;
				}>;
				if (!parsed.cid || !parsed.language) continue;
				out.push({
					cid: parsed.cid,
					language: parsed.language,
					display: parsed.display ?? parsed.language,
					release: parsed.release ?? null,
					externalId: parsed.externalId ?? '',
					season: typeof parsed.season === 'number' ? parsed.season : null,
					episode: typeof parsed.episode === 'number' ? parsed.episode : null
				});
			} catch {
				// pre-existing payloads without a `cid` (or non-JSON) — ignore
			}
		}
		return out;
	});

	// For TV firkins, restrict the subs visible to the player + picker to
	// the currently-playing episode so an S01E03 sub doesn't render on top
	// of S01E04 just because both live on the same firkin. Movies have
	// `season` / `episode` of `null` on every entry so the filter is a
	// no-op.
	const subsForCurrentEpisode = $derived<AttachedSubtitle[]>(
		isTmdbTv
			? attachedSubtitles.filter((s) => s.season === currentSeason && s.episode === currentEpisode)
			: attachedSubtitles.filter((s) => s.season === null && s.episode === null)
	);

	const playerSubtitles = $derived<PlayableFileSubtitle[]>(
		subsForCurrentEpisode.map((s) => ({
			languageCode: s.language,
			languageName: s.display,
			url: `${base}/api/ipfs/pins/${encodeURIComponent(s.cid)}/file`,
			isAutoGenerated: false
		}))
	);

	// Once a subtitle has been pinned and added to the firkin, push the
	// updated list onto the live player state so the in-page <select>
	// shows it without restarting the stream. Keyed off `firkin.id` so a
	// route swap doesn't accidentally inject the wrong sub list, and
	// guarded against unrelated player sessions (audio playback, other
	// firkins) so we don't clobber `currentFile.subtitles` for them.
	$effect(() => {
		const subs = playerSubtitles;
		const fid = firkin.id;
		const cur = get(playerService.state);
		const mode = get(playerService.displayMode);
		if (mode !== 'inline') return;
		if (cur.firkinId !== fid) return;
		if (!cur.currentFile) return;
		const existing = cur.currentFile.subtitles ?? [];
		const same =
			existing.length === subs.length && existing.every((e, i) => e.url === subs[i]?.url);
		if (same) return;
		playerService.state.update((s) => {
			if (!s.currentFile) return s;
			return { ...s, currentFile: { ...s.currentFile, subtitles: subs } };
		});
	});

	// Languages the user can pick from. Mirrors the backend filter in
	// `search_stremio_opensubs` (eng | cat | spa) — those are the only
	// codes the OpenSubtitles round-trip ever returns.
	const SUBTITLE_LANGUAGES: { code: string; label: string }[] = [
		{ code: 'eng', label: 'English' },
		{ code: 'cat', label: 'Catalan' },
		{ code: 'spa', label: 'Spanish' }
	];
	let selectedSubLanguage = $state<string>('eng');
	let selectedSubExternalId = $state<string | null>(null);
	let attachingSubtitle = $state<boolean>(false);
	let attachSubtitleError = $state<string | null>(null);

	const subsForLanguage = $derived.by<SubsLyricsItem[]>(() => {
		const lang = selectedSubLanguage;
		const all = subsLyricsResolver.results.filter(
			(r) => (r.language ?? '').toLowerCase() === lang.toLowerCase()
		);
		// When a torrent is attached, surface the best release-match first
		// so the dropdown's default option is the one most likely to be in
		// sync. Falls back to the upstream order for IPFS-only streams.
		if (firstMagnet) {
			const torrentTitle =
				firkin.files.find((f) => f.type === 'torrent magnet' && f.value === firstMagnet)?.title ??
				firkin.title;
			const matched = matchSubsToTorrent(torrentTitle ?? null, all);
			if (matched.length > 0) {
				const seen = new Set<SubsLyricsItem>();
				const ordered: SubsLyricsItem[] = [];
				for (const m of matched) {
					if (!seen.has(m.sub)) {
						seen.add(m.sub);
						ordered.push(m.sub);
					}
				}
				for (const s of all) {
					if (!seen.has(s)) ordered.push(s);
				}
				return ordered;
			}
		}
		return all;
	});

	// Drop the picker selection when the language changes (or when the
	// list re-sorts and the selected id is no longer present), so the
	// dropdown doesn't show a stale option label.
	$effect(() => {
		void selectedSubLanguage;
		if (selectedSubExternalId === null) return;
		const stillPresent = subsForLanguage.some((s) => s.externalId === selectedSubExternalId);
		if (!stillPresent) selectedSubExternalId = null;
	});

	async function attachSelectedSubtitle(): Promise<void> {
		if (attachingSubtitle) return;
		const id = selectedSubExternalId;
		if (!id) return;
		const pick = subsForLanguage.find((s) => s.externalId === id);
		if (!pick || !pick.url) return;
		// TV firkins must carry the (season, episode) on the persisted
		// FileEntry so the same OpenSubtitles externalId can land on
		// multiple episodes without dedup collisions, and so the player's
		// per-episode filter knows which sub belongs to which.
		if (isTmdbTv && (currentSeason === null || currentEpisode === null)) {
			attachSubtitleError = 'Start an episode stream before attaching a subtitle';
			return;
		}
		attachingSubtitle = true;
		attachSubtitleError = null;
		try {
			const payload: Parameters<typeof firkinsService.attachSubtitle>[1] = {
				source: pick.source,
				externalId: pick.externalId,
				url: pick.url,
				language: pick.language ?? selectedSubLanguage,
				display: pick.display ?? null,
				release: pick.release ?? null,
				format: pick.format ?? null,
				isHearingImpaired: pick.isHearingImpaired ?? false
			};
			if (isTmdbTv && currentSeason !== null && currentEpisode !== null) {
				payload.season = currentSeason;
				payload.episode = currentEpisode;
			}
			const updated = await firkinsService.attachSubtitle(firkin.id, payload);
			firkinOverride = updated;
		} catch (err) {
			attachSubtitleError = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			attachingSubtitle = false;
		}
	}

	const isSubtitleAttached = $derived(
		selectedSubExternalId !== null &&
			subsForCurrentEpisode.some(
				(s) =>
					s.externalId === selectedSubExternalId &&
					s.language.toLowerCase() === selectedSubLanguage.toLowerCase()
			)
	);
	const subPickerEnabled = $derived.by(() => {
		if (activeSource !== 'ipfs' && activeSource !== 'torrent') return false;
		if (!subsLyricsKind) return false;
		if (isTmdbTv) return currentSeason !== null && currentEpisode !== null;
		return true;
	});

	// Controlled active-subtitle URL fed to `<PlayerVideo subtitleUrl=…>`.
	// Resolves to the first attached subtitle whose language matches the
	// user's pick in the toolbar — and, on TV, also the currently-playing
	// `(season, episode)`. When the toolbar's specific release is also
	// attached, that wins (so a fresh download instantly shows on the
	// player even when an older same-language sub is still on the firkin).
	// Returns `null` to mean "subs off" — switching to a language with no
	// downloads yet hides any prior selection.
	const currentSubtitleUrl = $derived.by<string | null>(() => {
		const lang = selectedSubLanguage.toLowerCase();
		const candidates = subsForCurrentEpisode.filter((s) => s.language.toLowerCase() === lang);
		if (candidates.length === 0) return null;
		const preferred = selectedSubExternalId
			? candidates.find((s) => s.externalId === selectedSubExternalId)
			: undefined;
		const pick = preferred ?? candidates[candidates.length - 1];
		return `${base}/api/ipfs/pins/${encodeURIComponent(pick.cid)}/file`;
	});

	$effect(() => {
		if (!hasMagnetFiles || hasIpfsFiles) return;
		const id = firkin.id;
		let cancelled = false;
		const tick = async () => {
			if (cancelled) return;
			try {
				const res = await fetch(`${base}/api/firkins/${encodeURIComponent(id)}`, {
					cache: 'no-store'
				});
				if (cancelled) return;
				if (!res.ok) return;
				const fresh = (await res.json()) as Firkin;
				if (cancelled) return;
				if (fresh.files.some((f) => f.type === 'ipfs')) {
					firkinOverride = fresh;
				}
			} catch {
				// swallow — try again on next tick
			}
		};
		const timer = setInterval(tick, 4000);
		return () => {
			cancelled = true;
			clearInterval(timer);
		};
	});

	// Auto-start magnets that haven't been kicked yet. For tmdb-tv firkins
	// (which hold one magnet per season after a per-season fan-out search)
	// run them one-at-a-time: starting all of them in parallel saturates
	// the LAN, the disk, and the trackers, and the user typically watches
	// season-by-season anyway. The effect re-runs when `firkin.files`
	// changes (a new magnet is assigned) and when the polled torrent state
	// changes (a prior magnet finishes), so the next eligible magnet kicks
	// off automatically without an explicit completion hook. Other addons
	// keep their parallel-start behaviour — they never carry more than one
	// magnet in practice.
	$effect(() => {
		const magnets = firkin.files
			.filter((f) => f.type === 'torrent magnet' && f.value)
			.map((f) => f.value);
		if (magnets.length === 0) return;

		if (isTmdbTv) {
			// `started` covers two cases: (a) a torrent we've kicked this
			// session (startedHashes) — handles the up-to-2s window before
			// the poll picks it up, and (b) a torrent the manager already
			// knows about (poll result) — handles page reloads. `active`
			// is "started and not finished"; we only kick the next magnet
			// when nothing is active.
			const states = magnets.map((magnet) => {
				const hash = infoHashFromMagnet(magnet);
				const t = hash ? $torrentsState.byHash[hash] : null;
				const finished = !!t && (t.state === 'seeding' || t.progress >= 1);
				const started = startedHashes.has(magnet) || !!t;
				return { magnet, started, finished };
			});
			const active = states.find((s) => s.started && !s.finished);
			if (active) return;
			const next = states.find((s) => !s.started);
			if (!next) return;
			startedHashes = new Set(startedHashes).add(next.magnet);
			void startTorrentDownload(next.magnet).catch((err) => {
				console.warn('[catalog detail] auto-start failed for magnet:', err);
			});
			return;
		}

		for (const magnet of magnets) {
			if (startedHashes.has(magnet)) continue;
			startedHashes = new Set(startedHashes).add(magnet);
			void startTorrentDownload(magnet).catch((err) => {
				console.warn('[catalog detail] auto-start failed for magnet:', err);
			});
		}
	});

	/// Best-effort: promote a browse-cache firkin into a bookmarked one
	/// before persisting any torrent pick. Picking a torrent from the
	/// search table means the user is committing to this item, so the
	/// firkin should land in their library even if they never clicked the
	/// Bookmark button explicitly. Errors are logged and swallowed — the
	/// torrent action proceeds either way.
	async function ensureBookmarked(): Promise<void> {
		if (isBookmarked) return;
		try {
			const updated = await firkinsService.bookmark(firkin.id);
			firkinOverride = updated;
		} catch (err) {
			console.warn('[catalog detail] failed to auto-bookmark on torrent action:', err);
		}
	}

	async function assignTorrent(torrent: TorrentResultItem) {
		if (!torrent.magnetLink || addingHash || existingHashes.has(torrent.magnetLink)) {
			return;
		}
		assignError = null;
		addingHash = torrent.magnetLink;
		try {
			await ensureBookmarked();
			const additions: FileEntry[] = [
				{ type: 'torrent magnet', value: torrent.magnetLink, title: torrent.title }
			];
			if (isTmdbTv) {
				// TV shows accumulate one magnet per season on the same firkin.
				// Use the granular files endpoint so the server reads the
				// current `files` under the per-firkin lock and atomically
				// appends — without this, a concurrent persist (e.g. the
				// trailer resolver caching its preferred YouTube client)
				// could overwrite the magnet entry with a stale snapshot
				// and the torrent-completion watcher would never link the
				// downloaded episodes to this firkin.
				const updated = await firkinsService.mutateFiles(firkin.id, { add: additions });
				firkinOverride = updated;
			} else {
				// One-shot: build the full files array from the current
				// snapshot. This goes through `create` which dedupes by
				// CID, so the snapshot is intentional — if a different
				// snapshot also dedupes to the same CID, we get the
				// canonical record back and the user is redirected to it.
				const nextFiles = [...firkin.files, ...additions];
				const created = await firkinsService.create({
					title: firkin.title,
					artists: firkin.artists,
					description: firkin.description,
					images: firkin.images,
					files: nextFiles,
					year: firkin.year,
					addon: firkin.addon as FirkinAddon
				});
				await startTorrentDownload(torrent.magnetLink);
				if (created.id !== firkin.id) {
					await goto(`${base}/catalog/${encodeURIComponent(created.id)}`);
				}
			}
		} catch (err) {
			assignError = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			addingHash = null;
		}
	}

	/// Promote a browse-cache firkin into a bookmarked one. The server
	/// flips the flag in place (no CID roll, no version bump). On success
	/// the local firkin reactively gains its bookmarked surface — torrent
	/// search auto-fires, identity / version / files cards appear, and
	/// the action bar swaps over to Play / Find metadata / Delete.
	async function bookmark() {
		if (bookmarking || isBookmarked) return;
		bookmarking = true;
		bookmarkError = null;
		try {
			const updated = await firkinsService.bookmark(firkin.id);
			firkinOverride = updated;
		} catch (err) {
			bookmarkError = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			bookmarking = false;
		}
	}
</script>

<svelte:head>
	<title>Mhaol Cloud — {firkin.title}</title>
</svelte:head>

<div class="flex min-h-full flex-col gap-6 p-6">
	<CatalogPageHeader title={firkin.title} year={firkin.year} kindLabel={firkinKind}>
		{#snippet actions()}
			{#if canPlay}
				<button
					type="button"
					class="btn gap-2 btn-sm btn-primary"
					onclick={play}
					disabled={finalizing}
					aria-label="Play"
				>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						viewBox="0 0 24 24"
						fill="currentColor"
						stroke="none"
						class="h-4 w-4 shrink-0"
						aria-hidden="true"
					>
						<polygon points="6 4 20 12 6 20 6 4" />
					</svg>
					<span>{finalizing ? 'Pinning…' : 'Play'}</span>
				</button>
			{/if}
			{#if needsMetadata && lookupAddon}
				<button
					type="button"
					class="btn btn-outline btn-sm btn-info"
					onclick={() => (metadataLookupOpen = true)}
					title="Search {lookupAddon} and bake matching metadata into this firkin (rolls the version forward)"
				>
					Find metadata
				</button>
			{/if}
			<button
				type="button"
				class="btn gap-2 btn-sm btn-primary"
				onclick={bookmark}
				disabled={bookmarking || isBookmarked}
				aria-label="Bookmark"
				title={isBookmarked ? 'Already bookmarked' : 'Add this catalog item to your library'}
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					viewBox="0 0 24 24"
					fill="currentColor"
					stroke="none"
					class="h-4 w-4 shrink-0"
					aria-hidden="true"
				>
					<path d="M6 3h12a1 1 0 0 1 1 1v17l-7-4-7 4V4a1 1 0 0 1 1-1z" />
				</svg>
				<span>
					{#if bookmarking}
						Bookmarking…
					{:else if isBookmarked}
						Bookmarked
					{:else}
						Bookmark
					{/if}
				</span>
			</button>
		{/snippet}
	</CatalogPageHeader>

	{#if bookmarkError}
		<div class="alert alert-error"><span>{bookmarkError}</span></div>
	{/if}

	{#if finalizeError}
		<div class="alert alert-error"><span>{finalizeError}</span></div>
	{/if}
	{#if ipfsError}
		<div class="alert alert-error"><span>{ipfsError}</span></div>
	{/if}
	{#if torrentStreamError}
		<div class="alert alert-error"><span>{torrentStreamError}</span></div>
	{/if}

	<div class="grid grid-cols-1 gap-6 lg:grid-cols-4">
		<aside class="flex w-full flex-col gap-4">
			{#if firkin.images[0]}
				<img
					src={firkin.images[0].url}
					alt={firkin.title}
					loading="lazy"
					class="w-full rounded-md object-cover"
				/>
			{/if}

			<CatalogScoresCard reviews={firkin.reviews ?? []} />

			<FirkinArtistsSection
				artists={firkin.artists}
				loading={artistsBackfillStatus === 'loading'}
				error={artistsBackfillStatus === 'error' ? artistsBackfillError : null}
				emptyLabel="No people or groups attached. Re-bookmark from the catalog to enrich."
				artistHref={(id) => `${base}/artist/${encodeURIComponent(id)}`}
				singleColumn
			/>

			{#if isMusicBrainz}
				<CatalogAlbumsByArtistCard releaseGroupId={musicBrainzReleaseGroupId} />
			{/if}

			{#if isYoutubeVideo}
				<CatalogChannelLatestCard youtubeUrl={youtubeVideoUrl} />
			{/if}
		</aside>

		<section class="flex w-full flex-col gap-6 lg:col-span-2">
			{#if anyTabEnabled}
				<div class="flex flex-col gap-2">
					{#if subPickerEnabled}
						<div
							class="flex flex-wrap items-center gap-2 rounded border border-base-content/10 bg-base-200 px-2 py-1.5 text-xs"
						>
							<span class="font-semibold text-base-content/70 uppercase">Subtitles</span>
							<select
								class="select-bordered select select-xs"
								value={selectedSubLanguage}
								onchange={(e) => {
									selectedSubLanguage = (e.currentTarget as HTMLSelectElement).value;
								}}
								aria-label="Subtitle language"
							>
								{#each SUBTITLE_LANGUAGES as opt (opt.code)}
									<option value={opt.code}>{opt.label}</option>
								{/each}
							</select>
							<select
								class="select-bordered select min-w-0 flex-1 select-xs"
								value={selectedSubExternalId ?? ''}
								disabled={subsLyricsResolver.status === 'searching' || subsForLanguage.length === 0}
								onchange={(e) => {
									const v = (e.currentTarget as HTMLSelectElement).value;
									selectedSubExternalId = v === '' ? null : v;
								}}
								aria-label="Subtitle pick"
							>
								<option value=""
									>{subsLyricsResolver.status === 'searching'
										? 'Searching…'
										: subsForLanguage.length === 0
											? 'No matches'
											: 'Pick a subtitle…'}</option
								>
								{#each subsForLanguage as item (item.externalId)}
									<option value={item.externalId}>{item.release ?? `#${item.externalId}`}</option>
								{/each}
							</select>
							<button
								type="button"
								class="btn btn-xs btn-primary"
								disabled={!selectedSubExternalId || attachingSubtitle || isSubtitleAttached}
								onclick={attachSelectedSubtitle}
							>
								{#if attachingSubtitle}
									Downloading…
								{:else if isSubtitleAttached}
									Attached
								{:else}
									Use subtitle
								{/if}
							</button>
							{#if subsLyricsResolver.status === 'error'}
								<span class="text-error" title={subsLyricsResolver.error ?? ''}>Search failed</span>
							{/if}
							{#if attachSubtitleError}
								<span class="text-error" title={attachSubtitleError}>Attach failed</span>
							{/if}
						</div>
					{/if}

					{#snippet sourceButtons()}
						<button
							type="button"
							class="btn"
							disabled={!trailerTabEnabled}
							onclick={() => handleSourceClick('trailer')}
							title={trailerTabTitle}
						>
							{isYoutubeVideo ? 'Video' : 'Trailer'}
						</button>
						<button
							type="button"
							class="btn"
							disabled={!ipfsTabEnabled || ipfsStarting}
							onclick={() => handleSourceClick('ipfs')}
							title={ipfsTabTitle}
						>
							IPFS Stream{ipfsStarting ? ' — starting…' : ''}
						</button>
						<button
							type="button"
							class="btn"
							disabled={!torrentTabEnabled || torrentStreamStarting}
							onclick={() => handleSourceClick('torrent')}
							title={torrentTabTitle}
						>
							Torrent Stream{torrentTabSuffix}
						</button>
					{/snippet}

					{#if (activeSource === 'ipfs' || activeSource === 'torrent') && isInlinePlayingThisFirkin}
						<PlayerVideo
							file={$playerState.currentFile}
							connectionState={$playerState.connectionState}
							positionSecs={$playerState.positionSecs}
							durationSecs={$playerState.durationSecs}
							buffering={$playerState.buffering}
							poster={trailerThumb}
							hideSubtitleSelect
							subtitleUrl={currentSubtitleUrl}
							directStreamUrl={$playerState.directStreamUrl}
							directStreamMimeType={$playerState.directStreamMimeType}
							streamOffsetSecs={$playerState.streamOffsetSecs}
							onseekrequest={activeSource === 'ipfs' ? handleIpfsSeek : undefined}
							extraControls={sourceButtons}
						/>
					{:else if (activeSource === 'ipfs' && ipfsStarting) || (activeSource === 'torrent' && torrentStreamStarting)}
						<div
							class="flex aspect-video w-full items-center justify-center overflow-hidden rounded-md bg-black text-white"
						>
							<div class="text-center">
								<span class="loading loading-lg loading-spinner"></span>
								<p class="mt-2 text-sm opacity-70">
									Starting {activeSource === 'ipfs' ? 'IPFS' : 'torrent'} stream…
								</p>
							</div>
						</div>
					{:else if activeSource === 'trailer' && trailerTabEnabled}
						<CatalogTrailerPlayer
							posterUrl={trailerThumb}
							youtubeUrl={isYoutubeVideo ? youtubeVideoUrl : firstTrailerUrl}
							title={firkin.title}
							preferredClient={isYoutubeVideo ? youtubePreferredClient : null}
							onResolved={isYoutubeVideo ? persistYoutubePreferredClient : undefined}
							trailerOptions={isYoutubeVideo ? [] : playableTrailers}
							{selectedTrailerKey}
							onTrailerSelect={(k) => (selectedTrailerKey = k)}
							extraControls={sourceButtons}
						/>
					{:else if firkin.images[1]}
						<img
							src={firkin.images[1].url}
							alt={firkin.title}
							loading="lazy"
							class="w-full rounded-md object-cover"
						/>
					{/if}
				</div>
			{:else if firkin.images[1]}
				<img
					src={firkin.images[1].url}
					alt={firkin.title}
					loading="lazy"
					class="w-full rounded-md object-cover"
				/>
			{/if}

			<CatalogDescriptionPanel description={firkin.description} />

			{#if isTmdbMovie}
				<CatalogTorrentAttachmentCard
					download={downloadInfo}
					stream={streamInfo}
					onStreamPlay={replayStreamMagnet}
					streamPlaying={torrentStreamStarting}
					onDownloadPlay={playFromAttachmentDownload}
					downloadPlaying={ipfsStarting}
					preferredDownload={preferredDownloadTorrent}
					preferredStream={preferredStreamTorrent}
					attachingDownload={addingHash !== null &&
						preferredDownloadTorrent?.magnetLink === addingHash}
					attachingStream={streamingHash !== null &&
						preferredStreamTorrent?.magnetLink === streamingHash}
					onAttachDownload={assignTorrent}
					onAttachStream={streamTorrentFromRow}
				/>
			{/if}

			{#if isTmdbTv && tmdbTvId}
				<CatalogTvSeasonsCard
					{tmdbTvId}
					torrents={torrentSearch.matches}
					torrentsStatus={torrentSearch.status}
					torrentsError={torrentSearch.error}
					searchStack={torrentSearch.searchStack}
					{existingHashes}
					{addingHash}
					onAssign={assignTorrent}
					onSeasonsLoaded={(nums) => (tvSeasonNumbers = nums)}
					{availableEpisodeKeys}
					onPlayEpisode={playEpisode}
				/>
			{/if}

			{#if isMusicBrainz}
				<CatalogTracksCard
					resolver={trackResolver}
					{thumb}
					albumTitle={firkin.title}
					firkinId={firkin.id}
					onDownloadAlbum={startAlbumDownload}
					downloadInFlight={albumDownloadInFlight}
					downloadError={albumDownloadError}
				/>
			{/if}

			{#if isTmdbTv}
				{#if assignError}
					<div class="alert alert-error"><span>{assignError}</span></div>
				{/if}
			{:else if hasMagnetFiles}
				<CatalogTorrentSearchCard
					search={torrentSearch}
					onAssign={assignTorrent}
					onStream={streamTorrentFromRow}
					{addingHash}
					{streamingHash}
					{assignError}
					{existingHashes}
					collapsible
					open={torrentSearchOpen}
					onToggle={toggleTorrentSearch}
					onRefresh={() =>
						torrentSearch.search({
							addon: firkin.addon,
							title: firkin.title,
							year: firkin.year,
							firkinId: firkin.id,
							forceRefresh: true
						})}
				/>
			{:else if hasNoRealFiles && !isMusicBrainz}
				<CatalogTorrentSearchCard
					search={torrentSearch}
					onAssign={assignTorrent}
					onStream={streamTorrentFromRow}
					{addingHash}
					{streamingHash}
					{assignError}
					{existingHashes}
					collapsible
					open={torrentSearchOpen}
					onToggle={toggleTorrentSearch}
					onRefresh={() =>
						torrentSearch.search({
							addon: firkin.addon,
							title: firkin.title,
							year: firkin.year,
							firkinId: firkin.id,
							forceRefresh: true
						})}
				/>
			{/if}

			{#if subsLyricsKind}
				<CatalogSubsLyricsCard
					resolver={subsLyricsResolver}
					kind={subsLyricsKind}
					searchTerm={subsLyricsSearchTerm}
					onRefresh={runSubsLyricsSearch}
					collapsible
					open={subsLyricsOpen}
					onToggle={() => (subsLyricsOpen = !subsLyricsOpen)}
				/>
			{/if}

			{#if !isMusicBrainz}
				<CatalogFilesTable
					files={firkin.files}
					collapsible
					open={filesOpen}
					onToggle={() => (filesOpen = !filesOpen)}
				/>
			{/if}
		</section>

		<aside class="flex w-full flex-col gap-4">
			{#if isTmdbMovie}
				<CatalogRelatedCard
					addon={firkin.addon}
					upstreamId={tmdbMovieId}
					onItemsLoaded={handleRelatedItemsLoaded}
				/>
			{:else if isTmdbTv}
				<CatalogRelatedCard
					addon={firkin.addon}
					upstreamId={tmdbTvId}
					onItemsLoaded={handleRelatedItemsLoaded}
				/>
			{:else if isMusicBrainz}
				<CatalogRelatedCard
					addon={firkin.addon}
					upstreamId={musicBrainzReleaseGroupId}
					onItemsLoaded={handleRelatedItemsLoaded}
				/>
			{:else if isYoutubeVideo}
				<CatalogRelatedYoutubeCard youtubeUrl={youtubeVideoUrl} />
			{/if}
		</aside>
	</div>
</div>

{#if lookupAddon}
	<FirkinMetadataLookupModal
		open={metadataLookupOpen}
		addon={lookupAddon}
		initialQuery={firkin.title}
		firkinTitle={firkin.title}
		onpick={applyMetadata}
		onclose={() => (metadataLookupOpen = false)}
	/>
{/if}
