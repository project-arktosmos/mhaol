<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import { get } from 'svelte/store';
	import classNames from 'classnames';
	import FirkinArtistsSection from '$components/firkins/FirkinArtistsSection.svelte';
	import FirkinMetadataLookupModal, {
		type CatalogLookupItem
	} from '$components/firkins/FirkinMetadataLookupModal.svelte';
	import CatalogPageHeader from '$components/catalog/CatalogPageHeader.svelte';
	import CatalogDescriptionPanel from '$components/catalog/CatalogDescriptionPanel.svelte';
	import CatalogTrailersCard from '$components/catalog/CatalogTrailersCard.svelte';
	import CatalogTrailerPlayer from '$components/catalog/CatalogTrailerPlayer.svelte';
	import PlayerVideo from '$components/player/PlayerVideo.svelte';
	import CatalogTracksCard from '$components/catalog/CatalogTracksCard.svelte';
	import CatalogTorrentSearchCard from '$components/catalog/CatalogTorrentSearchCard.svelte';
	import CatalogTorrentProgressCard from '$components/catalog/CatalogTorrentProgressCard.svelte';
	import CatalogSubsLyricsCard from '$components/catalog/CatalogSubsLyricsCard.svelte';
	import CatalogRelatedCard from '$components/catalog/CatalogRelatedCard.svelte';
	import CatalogFilesTable from '$components/catalog/CatalogFilesTable.svelte';
	import { firkinPlaybackService } from '$services/firkin-playback.service';
	import { firkinTorrentsService, infoHashFromMagnet } from '$services/firkin-torrents.service';
	import { playerService } from '$services/player.service';
	import type { CloudFirkin } from '$types/firkin.type';
	import type { PlayableFile } from '$types/player.type';
	import {
		firkinsService,
		addonKind,
		metadataSearchAddon,
		type Firkin,
		type FirkinAddon
	} from '$lib/firkins.service';
	import { TrailerResolver } from '$services/catalog/trailer-resolver.svelte';
	import {
		TrackResolver,
		type AlbumProgressPayload
	} from '$services/catalog/track-resolver.svelte';
	import { TorrentSearch, startTorrentDownload } from '$services/catalog/torrent-search.svelte';
	import { SubsLyricsResolver } from '$services/catalog/subs-lyrics-resolver.svelte';
	import type { TorrentResultItem } from '$lib/search.service';
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
	const firkin = $derived<Firkin>(data.firkin);
	let removing = $state(false);
	let removeError = $state<string | null>(null);

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
	const thumb = $derived(firkin.images[0]?.url ?? null);
	// Trailers prefer the last image (typically the backdrop / wide art) so
	// the right-side player surfaces a 16:9 still rather than the poster.
	const trailerThumb = $derived(firkin.images[firkin.images.length - 1]?.url ?? thumb);

	const userIdentityState = userIdentityService.state;
	let recommendationsIngestedFor: string | null = null;

	function handleRelatedItemsLoaded(items: CatalogItem[]) {
		const sourceFirkinId = firkin.id;
		if (!sourceFirkinId) return;
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
		if (updated.id !== firkin.id) {
			void goto(`${base}/catalog/${encodeURIComponent(updated.id)}`);
		}
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

	$effect(() => {
		const fid = firkin.id;
		if (metadataBackfillForFirkinId === fid) return;
		const upstreamId = metadataUpstreamId();
		const reviewsMissing = (firkin.reviews ?? []).length === 0;
		const artistsMissing =
			firkin.artists.length === 0 && (isMusicBrainz || isTmdbMovie || isTmdbTv);
		if (!upstreamId || (!reviewsMissing && !artistsMissing)) {
			metadataBackfillForFirkinId = fid;
			return;
		}
		metadataBackfillForFirkinId = fid;
		void backfillFromMetadata(fid, firkin.addon, upstreamId, {
			fetchArtists: artistsMissing,
			fetchReviews: reviewsMissing
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
			data.firkin = updated;
			if (want.fetchArtists) artistsBackfillStatus = 'done';
			if (updated.id !== firkinId) {
				void goto(`${base}/catalog/${encodeURIComponent(updated.id)}`);
			}
		} catch (err) {
			artistsBackfillError = err instanceof Error ? err.message : 'Unknown error';
			if (want.fetchArtists) artistsBackfillStatus = 'error';
		}
	}

	async function startIpfsPlay(): Promise<void> {
		if (!firstIpfsCid || ipfsStarting) return;
		ipfsStarting = true;
		ipfsError = null;
		try {
			const res = await fetch(`${base}/api/ipfs-stream/sessions`, {
				method: 'POST',
				headers: { 'content-type': 'application/json' },
				body: JSON.stringify({ cid: firstIpfsCid })
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
				id: `firkin:${firkin.id}:ipfs:${firstIpfsCid}`,
				type: 'library',
				name: firkin.title,
				outputPath: '',
				mode: 'video',
				format: null,
				videoFormat: null,
				thumbnailUrl: thumb,
				durationSeconds: durationSecs,
				size: 0,
				completedAt: ''
			};
			await playerService.playUrl(
				file,
				body.playlistUrl,
				'application/vnd.apple.mpegurl',
				'inline',
				firkin.id
			);
		} catch (err) {
			ipfsError = err instanceof Error ? err.message : 'Unknown error';
			// Pipeline never produced a playlist — go back to the trailer tab
			// so the user isn't stuck on a permanent spinner. The error alert
			// above the player still surfaces what went wrong.
			if (activeSource === 'ipfs') activeSource = 'trailer';
		} finally {
			ipfsStarting = false;
		}
	}

	const torrentsState = firkinTorrentsService.state;
	onMount(() => firkinTorrentsService.start());

	const firstMagnet = $derived(
		firkin.files.find((f) => f.type === 'torrent magnet')?.value ?? null
	);

	let torrentStreamStarting = $state(false);
	let torrentStreamError = $state<string | null>(null);

	type StreamEvalState =
		| { kind: 'idle' }
		| { kind: 'evaluating' }
		| { kind: 'streamable'; fileName: string; fileSize: number; mimeType: string | null }
		| { kind: 'not-streamable'; reason: string };
	let streamEval = $state<StreamEvalState>({ kind: 'idle' });
	let evalRun = 0;

	$effect(() => {
		const magnet = firstMagnet;
		if (!magnet) {
			streamEval = { kind: 'idle' };
			return;
		}
		const myRun = ++evalRun;
		streamEval = { kind: 'evaluating' };
		void (async () => {
			try {
				const res = await fetch(`${base}/api/torrent/evaluate`, {
					method: 'POST',
					headers: { 'content-type': 'application/json' },
					body: JSON.stringify({ magnet })
				});
				if (myRun !== evalRun) return;
				const body = (await res.json()) as
					| {
							streamable: true;
							infoHash: string;
							name: string;
							fileIndex: number;
							fileName: string;
							fileSize: number;
							mimeType: string | null;
					  }
					| { streamable: false; reason: string };
				if (myRun !== evalRun) return;
				if (body.streamable) {
					streamEval = {
						kind: 'streamable',
						fileName: body.fileName,
						fileSize: body.fileSize,
						mimeType: body.mimeType
					};
				} else {
					streamEval = { kind: 'not-streamable', reason: body.reason };
				}
			} catch (err) {
				if (myRun !== evalRun) return;
				const message = err instanceof Error ? err.message : 'Unknown error';
				streamEval = { kind: 'not-streamable', reason: message };
			}
		})();
	});

	const trailerTabEnabled = $derived(isTmdbMovie || isTmdbTv);
	const ipfsTabEnabled = $derived(hasIpfsFiles);
	const torrentTabEnabled = $derived(streamEval.kind === 'streamable');
	const anyTabEnabled = $derived(trailerTabEnabled || ipfsTabEnabled || torrentTabEnabled);

	const trailerTabTitle = $derived(trailerTabEnabled ? 'Show trailer' : 'No trailer for this item');
	const ipfsTabTitle = $derived(
		ipfsTabEnabled
			? 'Stream over IPFS as HLS'
			: 'Available once at least one file is pinned to IPFS'
	);
	const torrentTabTitle = $derived.by(() => {
		if (!firstMagnet) return 'Available once a torrent magnet is attached';
		switch (streamEval.kind) {
			case 'idle':
			case 'evaluating':
				return 'Probing magnet metadata via DHT/trackers (BEP 9/10) — this can take 10–60s on DHT-only torrents';
			case 'not-streamable':
				return `Not streamable: ${streamEval.reason}`;
			case 'streamable':
				return `Stream "${streamEval.fileName}" as it downloads`;
		}
	});
	const torrentTabSuffix = $derived.by(() => {
		if (torrentStreamStarting) return ' — starting…';
		switch (streamEval.kind) {
			case 'idle':
			case 'evaluating':
				return ' — probing…';
			case 'not-streamable':
				return ' — unavailable';
			case 'streamable':
				return '';
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

	async function startTorrentStream(): Promise<void> {
		if (!firstMagnet || torrentStreamStarting) return;
		torrentStreamStarting = true;
		torrentStreamError = null;
		try {
			const res = await fetch(`${base}/api/torrent/stream`, {
				method: 'POST',
				headers: { 'content-type': 'application/json' },
				body: JSON.stringify({ magnet: firstMagnet })
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
				completedAt: ''
			};
			await playerService.playUrl(file, body.streamUrl, body.mimeType ?? null, 'inline', firkin.id);
		} catch (err) {
			torrentStreamError = err instanceof Error ? err.message : 'Unknown error';
			if (activeSource === 'torrent') activeSource = 'trailer';
		} finally {
			torrentStreamStarting = false;
		}
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

	const torrentProgressRows = $derived.by(() => {
		const seen = new Set<string>();
		const rows: { title: string | null; torrent: (typeof $torrentsState.byHash)[string] }[] = [];
		for (const f of firkin.files) {
			if (f.type !== 'torrent magnet' || !f.value) continue;
			const hash = infoHashFromMagnet(f.value);
			if (!hash || seen.has(hash)) continue;
			const t = $torrentsState.byHash[hash];
			if (!t) continue;
			seen.add(hash);
			rows.push({ title: f.title ?? null, torrent: t });
		}
		return rows;
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
			if (next.id !== firkin.id) {
				await goto(`${base}/catalog/${encodeURIComponent(next.id)}`);
			} else {
				data.firkin = next;
			}
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
	// don't have to be re-resolved every visit. The PUT may roll the firkin
	// forward to a new content-addressed id; in that case we navigate.
	async function persistFirkinPatch(patch: Partial<Firkin>): Promise<void> {
		const oldId = firkin.id;
		const res = await fetch(`${base}/api/firkins/${encodeURIComponent(oldId)}`, {
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
		data.firkin = updated;
		if (updated.id !== oldId) {
			void goto(`${base}/catalog/${encodeURIComponent(updated.id)}`);
		}
	}

	const trailerResolver = new TrailerResolver({
		persist: (resolved) => persistFirkinPatch({ trailers: resolved })
	});
	// First playable trailer URL — drives the inline `CatalogTrailerPlayer`
	// above the description (replacing the second image). Stays null until
	// the resolver finds a YouTube URL; the player keeps showing the poster
	// in the meantime so the area never appears blank.
	const firstTrailerUrl = $derived(
		trailerResolver.trailers.find((t) => Boolean(t.youtubeUrl))?.youtubeUrl ?? null
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
	// running, poll the firkin every few seconds. When it rolls forward
	// to a new content-addressed id, navigate to it; if the body comes
	// back at the same id with new files (rare — only when no rollforward
	// was needed), refresh the in-memory copy and re-project tracks.
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
				if (res.status === 404) {
					const listRes = await fetch(`${base}/api/firkins`, { cache: 'no-store' });
					if (!listRes.ok) return;
					const list = (await listRes.json()) as Firkin[];
					if (cancelled) return;
					const successor = list.find((d) => (d.version_hashes ?? []).includes(id));
					if (successor) {
						await goto(`${base}/catalog/${encodeURIComponent(successor.id)}`);
					}
					return;
				}
				if (!res.ok) return;
				const fresh = (await res.json()) as Firkin;
				if (cancelled) return;
				if (fresh.id !== id) {
					await goto(`${base}/catalog/${encodeURIComponent(fresh.id)}`);
					return;
				}
				const freshHasMore =
					fresh.files.length !== firkin.files.length || fresh.updated_at !== firkin.updated_at;
				if (freshHasMore) {
					data.firkin = fresh;
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

	const torrentSearch = new TorrentSearch({ evaluate: true });
	let addingHash = $state<string | null>(null);
	let assignError = $state<string | null>(null);
	let startedHashes = $state<Set<string>>(new Set());
	let torrentSearchOpen = $state(false);
	let torrentSearchInitForFirkinId: string | null = null;

	const existingHashes = $derived(
		new Set(firkin.files.filter((f) => f.type === 'torrent magnet' && f.value).map((f) => f.value))
	);

	function toggleTorrentSearch() {
		torrentSearchOpen = !torrentSearchOpen;
		if (torrentSearchOpen && torrentSearchInitForFirkinId !== firkin.id) {
			torrentSearchInitForFirkinId = firkin.id;
			void torrentSearch.search({ addon: firkin.addon, title: firkin.title, year: firkin.year });
		}
	}

	// When the firkin is just bookmarked metadata (no IPFS files, no
	// magnets), kick the torrent search off automatically so the user can
	// pick a source without having to click into a collapsed card. Mirrors
	// the auto-search the virtual page already does. MusicBrainz is
	// excluded because albums get the tracks card, not torrents.
	$effect(() => {
		if (isMusicBrainz) return;
		if (!hasNoRealFiles) return;
		if (torrentSearchInitForFirkinId === firkin.id) return;
		torrentSearchInitForFirkinId = firkin.id;
		void torrentSearch.search({ addon: firkin.addon, title: firkin.title, year: firkin.year });
	});

	const subsLyricsResolver = new SubsLyricsResolver();
	const subsLyricsKind = $derived<'subs' | null>(isTmdbMovie || isTmdbTv ? 'subs' : null);
	let subsLyricsInitForFirkinId: string | null = null;

	function runSubsLyricsSearch() {
		if (!subsLyricsKind) return;
		const externalId = isTmdbMovie ? tmdbMovieId : tmdbTvId;
		if (!externalId) return;
		void subsLyricsResolver.search({
			addon: firkin.addon,
			query: firkin.title,
			externalIds: [externalId]
		});
	}

	$effect(() => {
		if (!subsLyricsKind) return;
		if (!(isTmdbMovie ? tmdbMovieId : tmdbTvId)) return;
		const fid = firkin.id;
		if (subsLyricsInitForFirkinId === fid) return;
		subsLyricsInitForFirkinId = fid;
		runSubsLyricsSearch();
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
				if (res.status === 404) {
					const listRes = await fetch(`${base}/api/firkins`, { cache: 'no-store' });
					if (!listRes.ok) return;
					const list = (await listRes.json()) as Firkin[];
					if (cancelled) return;
					const successor = list.find((d) => (d.version_hashes ?? []).includes(id));
					if (successor) {
						await goto(`${base}/catalog/${encodeURIComponent(successor.id)}`);
					}
					return;
				}
				if (!res.ok) return;
				const fresh = (await res.json()) as Firkin;
				if (cancelled) return;
				if (fresh.files.some((f) => f.type === 'ipfs')) {
					data.firkin = fresh;
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

	$effect(() => {
		const magnets = firkin.files
			.filter((f) => f.type === 'torrent magnet' && f.value)
			.map((f) => f.value);
		for (const magnet of magnets) {
			if (startedHashes.has(magnet)) continue;
			startedHashes = new Set(startedHashes).add(magnet);
			void startTorrentDownload(magnet).catch((err) => {
				console.warn('[catalog detail] auto-start failed for magnet:', err);
			});
		}
	});

	async function assignTorrent(torrent: TorrentResultItem) {
		if (!torrent.magnetLink || addingHash || existingHashes.has(torrent.magnetLink)) {
			return;
		}
		assignError = null;
		addingHash = torrent.magnetLink;
		try {
			const created = await firkinsService.create({
				title: firkin.title,
				artists: firkin.artists,
				description: firkin.description,
				images: firkin.images,
				files: [
					...firkin.files,
					{ type: 'torrent magnet', value: torrent.magnetLink, title: torrent.title }
				],
				year: firkin.year,
				addon: firkin.addon as FirkinAddon
			});
			await startTorrentDownload(torrent.magnetLink);
			if (created.id !== firkin.id) {
				await goto(`${base}/catalog/${encodeURIComponent(created.id)}`);
			}
		} catch (err) {
			assignError = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			addingHash = null;
		}
	}

	async function remove() {
		if (removing) return;
		removing = true;
		removeError = null;
		try {
			await firkinsService.remove(firkin.id);
			window.location.href = `${base}/catalog`;
		} catch (err) {
			removeError = err instanceof Error ? err.message : 'Unknown error';
			removing = false;
		}
	}
</script>

<svelte:head>
	<title>Mhaol Cloud — {firkin.title}</title>
</svelte:head>

<div class="flex min-h-full flex-col gap-6 p-6">
	<CatalogPageHeader
		title={firkin.title}
		addon={firkin.addon}
		kindLabel={firkinKind}
		year={firkin.year}
	>
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
				class="btn btn-outline btn-sm btn-error"
				onclick={remove}
				disabled={removing}
			>
				{removing ? 'Deleting…' : 'Delete firkin'}
			</button>
		{/snippet}
	</CatalogPageHeader>

	{#if removeError}
		<div class="alert alert-error"><span>{removeError}</span></div>
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

	<div class="grid grid-cols-1 gap-6 lg:grid-cols-[minmax(0,_320px)_1fr]">
		<aside class="flex flex-col gap-4">
			{#if firkin.images[0]}
				<img
					src={firkin.images[0].url}
					alt={firkin.title}
					loading="lazy"
					class="w-full rounded-md object-cover"
				/>
			{/if}

			<FirkinArtistsSection
				artists={firkin.artists}
				loading={artistsBackfillStatus === 'loading'}
				error={artistsBackfillStatus === 'error' ? artistsBackfillError : null}
				emptyLabel="No people or groups attached. Re-bookmark from the catalog to enrich."
				artistHref={(id) => `${base}/artist/${encodeURIComponent(id)}`}
				singleColumn
			/>
		</aside>

		<section class="flex flex-col gap-6">
			{#if anyTabEnabled}
				<div class="flex flex-col gap-2">
					<div role="tablist" class="tabs-bordered tabs">
						<button
							type="button"
							role="tab"
							class={classNames('tab', { 'tab-active': activeSource === 'trailer' })}
							disabled={!trailerTabEnabled}
							onclick={() => selectSource('trailer')}
							title={trailerTabTitle}
						>
							Trailer
						</button>
						<button
							type="button"
							role="tab"
							class={classNames('tab', { 'tab-active': activeSource === 'ipfs' })}
							disabled={!ipfsTabEnabled || ipfsStarting}
							onclick={() => selectSource('ipfs')}
							title={ipfsTabTitle}
						>
							IPFS Stream{ipfsStarting ? ' — starting…' : ''}
						</button>
						<button
							type="button"
							role="tab"
							class={classNames('tab', { 'tab-active': activeSource === 'torrent' })}
							disabled={!torrentTabEnabled || torrentStreamStarting}
							onclick={() => selectSource('torrent')}
							title={torrentTabTitle}
						>
							Torrent Stream{torrentTabSuffix}
						</button>
					</div>

					{#if (activeSource === 'ipfs' || activeSource === 'torrent') && isInlinePlayingThisFirkin}
						<PlayerVideo
							file={$playerState.currentFile}
							connectionState={$playerState.connectionState}
							positionSecs={$playerState.positionSecs}
							durationSecs={$playerState.durationSecs}
							buffering={$playerState.buffering}
							poster={trailerThumb}
							directStreamUrl={$playerState.directStreamUrl}
							directStreamMimeType={$playerState.directStreamMimeType}
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
							youtubeUrl={firstTrailerUrl}
							title={firkin.title}
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

			<CatalogDescriptionPanel
				description={firkin.description}
				identity={{
					cid: firkin.id,
					createdAt: firkin.created_at,
					updatedAt: firkin.updated_at,
					version: firkin.version ?? 0
				}}
				versionHashes={firkin.version_hashes ?? []}
				reviews={firkin.reviews ?? []}
			/>

			<CatalogTorrentProgressCard rows={torrentProgressRows} />

			{#if isTmdbMovie || isTmdbTv}
				<CatalogTrailersCard resolver={trailerResolver} firkinTitle={firkin.title} {thumb} />
			{/if}

			{#if isMusicBrainz}
				<CatalogTracksCard resolver={trackResolver} {thumb} albumTitle={firkin.title} />
			{/if}

			{#if hasMagnetFiles}
				<CatalogTorrentSearchCard
					search={torrentSearch}
					onAssign={assignTorrent}
					{addingHash}
					{assignError}
					{existingHashes}
					collapsible
					open={torrentSearchOpen}
					onToggle={toggleTorrentSearch}
					onRefresh={() =>
						torrentSearch.search({
							addon: firkin.addon,
							title: firkin.title,
							year: firkin.year
						})}
				/>
			{:else if hasNoRealFiles && !isMusicBrainz}
				<CatalogTorrentSearchCard
					search={torrentSearch}
					onAssign={assignTorrent}
					{addingHash}
					{assignError}
					{existingHashes}
					onRefresh={() =>
						torrentSearch.search({
							addon: firkin.addon,
							title: firkin.title,
							year: firkin.year
						})}
				/>
			{/if}

			{#if subsLyricsKind}
				<CatalogSubsLyricsCard
					resolver={subsLyricsResolver}
					kind={subsLyricsKind}
					onRefresh={runSubsLyricsSearch}
				/>
			{/if}

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
			{/if}

			<CatalogFilesTable files={firkin.files} />
		</section>
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
