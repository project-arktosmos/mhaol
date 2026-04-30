<script lang="ts">
	import { onMount } from 'svelte';
	import classNames from 'classnames';
	import FirkinCard from 'ui-lib/components/firkins/FirkinCard.svelte';
	import { firkinPlaybackService } from 'ui-lib/services/firkin-playback.service';
	import {
		firkinTorrentsService,
		infoHashFromMagnet
	} from 'ui-lib/services/firkin-torrents.service';
	import { playerService } from 'ui-lib/services/player.service';
	import type { CloudFirkin } from 'ui-lib/types/firkin.type';
	import type { PlayableFile } from 'ui-lib/types/player.type';
	import { cachedImageUrl } from '$lib/image-cache';
	import {
		firkinsService,
		addonKind,
		type Firkin,
		type FirkinAddon,
		type FileEntry
	} from '$lib/firkins.service';
	import {
		formatSizeBytes,
		matchTorrentsForResult,
		searchTorrents,
		type TorrentResultItem
	} from '$lib/search.service';
	import { playYouTubeAudio, resolveYouTubeUrlForTrack } from '$lib/youtube-match.service';
	import { base } from '$app/paths';
	import { goto } from '$app/navigation';

	interface Props {
		data: { firkin: Firkin };
	}

	let { data }: Props = $props();
	const firkin = $derived<Firkin>(data.firkin);
	let removing = $state(false);
	let removeError = $state<string | null>(null);

	const hasIpfsFiles = $derived(firkin.files.some((f) => f.type === 'ipfs'));
	const firstIpfsCid = $derived(firkin.files.find((f) => f.type === 'ipfs')?.value ?? null);
	const hasMagnetFiles = $derived(firkin.files.some((f) => f.type === 'torrent magnet'));
	const firkinKind = $derived(addonKind(firkin.addon));
	const isMusicBrainz = $derived(firkin.addon === 'musicbrainz');

	function isYouTubeUrl(value: string): boolean {
		try {
			const host = new URL(value).hostname.toLowerCase();
			return (
				host === 'www.youtube.com' ||
				host === 'youtube.com' ||
				host === 'm.youtube.com' ||
				host === 'music.youtube.com' ||
				host === 'youtu.be'
			);
		} catch {
			return false;
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

	// Track entries are persisted as `url` files whose value points at YouTube.
	// Other `url` files (e.g. the MusicBrainz release-group source URL stored
	// at bookmark time) must be excluded so they don't render as fake tracks.
	const trackFiles = $derived(
		firkin.files.filter(
			(f) => f.type === 'url' && (f.title ?? '').trim().length > 0 && isYouTubeUrl(f.value)
		)
	);

	const musicBrainzReleaseGroupId = $derived(
		firkin.files
			.map((f) => (f.type === 'url' ? parseMusicBrainzReleaseGroupId(f.value) : null))
			.find((id): id is string => Boolean(id)) ?? null
	);
	const isStreamUrlKind = $derived(firkinKind === 'iptv channel' || firkinKind === 'radio station');
	const firstStreamUrl = $derived(
		isStreamUrlKind ? (firkin.files.find((f) => f.type === 'url')?.value ?? null) : null
	);
	const hasStreamUrl = $derived(firstStreamUrl !== null);
	let ipfsStarting = $state(false);
	let ipfsError = $state<string | null>(null);

	async function startIpfsPlay(): Promise<void> {
		if (!firstIpfsCid || ipfsStarting) return;
		ipfsStarting = true;
		ipfsError = null;
		try {
			const res = await fetch('/api/ipfs-stream/sessions', {
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
				thumbnailUrl: firkin.images[0]?.url ?? null,
				durationSeconds: durationSecs,
				size: 0,
				completedAt: ''
			};
			// The rolling HLS playlist has no #EXT-X-ENDLIST until transcode
			// completes, so videoElement.duration stays Infinity — seed
			// durationSecs from the server-probed source duration instead.
			await playerService.playUrl(
				file,
				body.playlistUrl,
				'application/vnd.apple.mpegurl',
				'sidebar'
			);
		} catch (err) {
			ipfsError = err instanceof Error ? err.message : 'Unknown error';
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
		| {
				kind: 'streamable';
				fileName: string;
				fileSize: number;
				mimeType: string | null;
		  }
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
				const res = await fetch('/api/torrent/evaluate', {
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

	const torrentStreamButtonDisabled = $derived(
		!firstMagnet ||
			streamEval.kind === 'idle' ||
			streamEval.kind === 'evaluating' ||
			streamEval.kind === 'not-streamable' ||
			torrentStreamStarting
	);
	const torrentStreamButtonTitle = $derived.by(() => {
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
	const torrentStreamButtonLabel = $derived.by(() => {
		if (torrentStreamStarting) return 'Starting…';
		switch (streamEval.kind) {
			case 'idle':
				return 'Torrent Stream';
			case 'evaluating':
				return 'Probing…';
			case 'not-streamable':
				return 'Not streamable';
			case 'streamable':
				return 'Torrent Stream';
		}
	});

	async function startTorrentStream(): Promise<void> {
		if (!firstMagnet || torrentStreamStarting) return;
		torrentStreamStarting = true;
		torrentStreamError = null;
		try {
			const res = await fetch('/api/torrent/stream', {
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
				thumbnailUrl: firkin.images[0]?.url ?? null,
				durationSeconds: null,
				size: body.fileSize,
				completedAt: ''
			};
			await playerService.playUrl(file, body.streamUrl, body.mimeType ?? null, 'sidebar');
		} catch (err) {
			torrentStreamError = err instanceof Error ? err.message : 'Unknown error';
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

	const isRetroAchievementsConsole = $derived(firkin.addon === 'retroachievements');

	type RomReport = {
		torrent_paths: string[];
		archives: { name: string; relative_path: string; status: string; error?: string }[];
		roms: { name: string; relative_path: string; size: number }[];
	};
	type RomsStatus = 'idle' | 'loading' | 'done' | 'error';
	let romsStatus = $state<RomsStatus>('idle');
	let romsError = $state<string | null>(null);
	let roms = $state<RomReport | null>(null);
	let romsRun = 0;
	let romsLoadedForKey: string | null = null;

	async function loadRoms(force = false): Promise<void> {
		const key = `${firkin.id}:${completedTorrents.map((t) => t.hash).join(',')}`;
		if (!force && romsLoadedForKey === key) return;
		romsLoadedForKey = key;
		const myRun = ++romsRun;
		romsStatus = 'loading';
		romsError = null;
		try {
			const res = await fetch(`/api/firkins/${encodeURIComponent(firkin.id)}/roms`, {
				cache: 'no-store'
			});
			if (myRun !== romsRun) return;
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
			const body = (await res.json()) as RomReport;
			if (myRun !== romsRun) return;
			roms = body;
			romsStatus = 'done';
		} catch (err) {
			if (myRun !== romsRun) return;
			romsError = err instanceof Error ? err.message : 'Unknown error';
			romsStatus = 'error';
		}
	}

	$effect(() => {
		if (!isRetroAchievementsConsole) return;
		if (completedTorrents.length === 0) return;
		void loadRoms(false);
	});

	const canPlay = $derived(hasIpfsFiles || completedTorrents.length > 0 || hasStreamUrl);

	let finalizing = $state(false);
	let finalizeError = $state<string | null>(null);

	async function play() {
		if (hasStreamUrl && firstStreamUrl) {
			const mode: 'audio' | 'video' = firkinKind === 'radio station' ? 'audio' : 'video';
			const file: PlayableFile = {
				id: `firkin:${firkin.id}`,
				type: 'library',
				name: firkin.title,
				outputPath: '',
				mode,
				format: null,
				videoFormat: null,
				thumbnailUrl: firkin.images[0]?.url ?? null,
				durationSeconds: null,
				size: 0,
				completedAt: ''
			};
			const mime = firkinKind === 'iptv channel' ? 'application/vnd.apple.mpegurl' : null;
			await playerService.playUrl(file, firstStreamUrl, mime, 'sidebar');
			return;
		}
		if (hasIpfsFiles) {
			firkinPlaybackService.select(firkin as CloudFirkin);
			return;
		}
		if (finalizing) return;
		finalizeError = null;
		finalizing = true;
		try {
			const res = await fetch(`/api/firkins/${encodeURIComponent(firkin.id)}/finalize`, {
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

	type TorrentStatus = 'idle' | 'searching' | 'done' | 'error';
	let torrentStatus = $state<TorrentStatus>('idle');
	let torrentError = $state<string | null>(null);
	let torrentMatches = $state<TorrentResultItem[]>([]);
	let addingHash = $state<string | null>(null);
	let assignError = $state<string | null>(null);
	let searchRun = 0;
	let startedHashes = $state<Set<string>>(new Set());

	type TorrentRowEval =
		| { kind: 'pending' }
		| { kind: 'evaluating' }
		| { kind: 'streamable'; fileName: string; fileSize: number; mimeType: string | null }
		| { kind: 'not-streamable'; reason: string };
	let resultEvals = $state<Record<string, TorrentRowEval>>({});
	const EVAL_CONCURRENCY = 4;

	const existingHashes = $derived(
		new Set(firkin.files.filter((f) => f.type === 'torrent magnet' && f.value).map((f) => f.value))
	);

	let torrentSearchOpen = $state(false);
	let torrentSearchInitForFirkinId: string | null = null;

	function toggleTorrentSearch() {
		torrentSearchOpen = !torrentSearchOpen;
		if (torrentSearchOpen && torrentSearchInitForFirkinId !== firkin.id) {
			torrentSearchInitForFirkinId = firkin.id;
			void runTorrentSearch(firkin.id, firkin.title, firkin.addon, firkin.year);
		}
	}

	type Track = {
		id: string;
		position: number;
		title: string;
		lengthMs: number | null;
		youtubeUrl: string | null;
		youtubeStatus: 'idle' | 'pending' | 'searching' | 'found' | 'missing' | 'error';
	};
	type TracksStatus = 'idle' | 'loading' | 'done' | 'error';
	let tracksStatus = $state<TracksStatus>('idle');
	let tracksError = $state<string | null>(null);
	let tracks = $state<Track[]>([]);
	let tracksRun = 0;
	let tracksInitForFirkinId: string | null = null;

	$effect(() => {
		if (!isMusicBrainz) return;
		const fid = firkin.id;
		if (tracksInitForFirkinId === fid) return;
		tracksInitForFirkinId = fid;
		const myRun = ++tracksRun;
		if (trackFiles.length > 0) {
			tracks = trackFiles.map((f, i) => ({
				id: `file-${i}`,
				position: i + 1,
				title: f.title ?? '',
				lengthMs: null,
				youtubeUrl: f.value || null,
				youtubeStatus: f.value ? 'idle' : 'pending'
			}));
			tracksStatus = 'done';
			void resolveYouTubeForAllTracks(myRun);
		} else if (musicBrainzReleaseGroupId) {
			void loadByReleaseGroupId(musicBrainzReleaseGroupId, myRun);
		} else {
			tracksError =
				'No MusicBrainz release-group id stored on this firkin. Re-bookmark from the catalog to attach one.';
			tracksStatus = 'error';
		}
	});

	async function loadByReleaseGroupId(releaseGroupId: string, myRun: number) {
		tracksStatus = 'loading';
		tracksError = null;
		tracks = [];
		try {
			const res = await fetch(
				`/api/catalog/musicbrainz/release-groups/${encodeURIComponent(releaseGroupId)}/tracks`,
				{ cache: 'no-store' }
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
			const body = (await res.json()) as {
				id: string;
				position: number;
				title: string;
				lengthMs: number | null;
			}[];
			if (myRun !== tracksRun) return;
			tracks = body.map((t) => ({
				id: t.id,
				position: t.position,
				title: t.title,
				lengthMs: t.lengthMs,
				youtubeUrl: null,
				youtubeStatus: 'pending' as const
			}));
			tracksStatus = 'done';
			void resolveYouTubeForAllTracks(myRun);
		} catch (err) {
			if (myRun !== tracksRun) return;
			tracksError = err instanceof Error ? err.message : 'Unknown error';
			tracksStatus = 'error';
		}
	}

	async function resolveYouTubeForAllTracks(myRun: number) {
		const album = firkin.title;
		const artist = firkin.artists
			.map((a) => a.name)
			.filter((n) => n && n.length > 0)
			.join(', ');
		// Snapshot the firkin's files once and thread the accumulator through
		// the loop. Re-reading `firkin.files` between sequential PUTs is
		// unsafe: the previous iteration's `data.firkin = updated` does not
		// reliably propagate through the `$derived` before the next snapshot,
		// so each PUT was being built from stale files and clobbering the
		// previous PUT — leaving only the last-resolved track on the firkin.
		let workingFiles: FileEntry[] = firkin.files.map((f) => ({ ...f }));
		for (let i = 0; i < tracks.length; i++) {
			if (myRun !== tracksRun) return;
			const t = tracks[i];
			if (t.youtubeStatus === 'idle' || t.youtubeStatus === 'found') continue;
			tracks = tracks.map((tr, idx) => (idx === i ? { ...tr, youtubeStatus: 'searching' } : tr));
			let url: string | null = null;
			try {
				url = await resolveYouTubeUrlForTrack(t.title, artist, album, t.lengthMs);
				if (myRun !== tracksRun) return;
				tracks = tracks.map((tr, idx) =>
					idx === i ? { ...tr, youtubeUrl: url, youtubeStatus: url ? 'found' : 'missing' } : tr
				);
			} catch {
				if (myRun !== tracksRun) return;
				tracks = tracks.map((tr, idx) =>
					idx === i ? { ...tr, youtubeUrl: null, youtubeStatus: 'error' } : tr
				);
				continue;
			}
			if (url) {
				try {
					workingFiles = await persistTrackUrl(workingFiles, t.title, url);
				} catch (err) {
					console.warn('[catalog detail] failed to persist track url', err);
				}
			}
		}
	}

	async function persistTrackUrl(
		currentFiles: FileEntry[],
		trackTitle: string,
		url: string
	): Promise<FileEntry[]> {
		const next = currentFiles.map((f) => ({ ...f }));
		const idx = next.findIndex(
			(f) => f.type === 'url' && (f.title ?? '').trim() === trackTitle.trim()
		);
		if (idx >= 0) {
			next[idx] = { ...next[idx], value: url };
		} else {
			next.push({ type: 'url', value: url, title: trackTitle });
		}
		const res = await fetch(`/api/firkins/${encodeURIComponent(firkin.id)}`, {
			method: 'PUT',
			headers: { 'content-type': 'application/json' },
			body: JSON.stringify({ files: next })
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
		return updated.files;
	}

	function formatDuration(ms: number | null): string {
		if (!ms || !Number.isFinite(ms) || ms <= 0) return '—';
		const total = Math.round(ms / 1000);
		const m = Math.floor(total / 60);
		const s = total % 60;
		return `${m}:${s.toString().padStart(2, '0')}`;
	}

	let playingTrackIndex = $state<number | null>(null);
	let trackPlayError = $state<string | null>(null);

	async function playTrack(index: number) {
		const t = tracks[index];
		if (!t || !t.youtubeUrl || playingTrackIndex !== null) return;
		playingTrackIndex = index;
		trackPlayError = null;
		try {
			const durationSeconds = t.lengthMs ? Math.round(t.lengthMs / 1000) : null;
			const thumb = firkin.images[0]?.url ?? null;
			await playYouTubeAudio(t.youtubeUrl, t.title, thumb, durationSeconds);
		} catch (err) {
			trackPlayError = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			playingTrackIndex = null;
		}
	}

	$effect(() => {
		if (!hasMagnetFiles || hasIpfsFiles) return;
		const id = firkin.id;
		let cancelled = false;
		const tick = async () => {
			if (cancelled) return;
			try {
				const res = await fetch(`/api/firkins/${encodeURIComponent(id)}`, {
					cache: 'no-store'
				});
				if (cancelled) return;
				if (res.status === 404) {
					const listRes = await fetch('/api/firkins', { cache: 'no-store' });
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

	async function runTorrentSearch(_id: string, title: string, addon: string, year: number | null) {
		const myRun = ++searchRun;
		torrentStatus = 'searching';
		torrentError = null;
		torrentMatches = [];
		resultEvals = {};
		try {
			const torrents = await searchTorrents(addon, title);
			if (myRun !== searchRun) return;
			const matches = matchTorrentsForResult(
				{ title, description: '', artists: [], images: [], files: [], year, raw: null },
				torrents
			);
			torrentMatches = matches;
			torrentStatus = 'done';
			void evaluateResults(matches, myRun);
		} catch (err) {
			if (myRun !== searchRun) return;
			torrentMatches = [];
			torrentError = err instanceof Error ? err.message : 'Unknown error';
			torrentStatus = 'error';
		}
	}

	async function evaluateResults(matches: TorrentResultItem[], runToken: number): Promise<void> {
		const seed: Record<string, TorrentRowEval> = {};
		for (const t of matches) {
			if (t.magnetLink) seed[t.magnetLink] = { kind: 'pending' };
		}
		resultEvals = seed;

		// Sliding-window concurrency: each `/api/torrent/evaluate` call may
		// block for up to ~60s while librqbit fetches metadata via DHT or
		// trackers (BEP 9/10). Firing all of them at once would saturate the
		// torrent client; cap to a small fixed pool instead.
		let cursor = 0;
		const next = (): TorrentResultItem | null => {
			while (cursor < matches.length) {
				const t = matches[cursor++];
				if (t.magnetLink) return t;
			}
			return null;
		};

		const worker = async () => {
			while (runToken === searchRun) {
				const t = next();
				if (!t || !t.magnetLink) break;
				resultEvals = { ...resultEvals, [t.magnetLink]: { kind: 'evaluating' } };
				let result: TorrentRowEval;
				try {
					const res = await fetch('/api/torrent/evaluate', {
						method: 'POST',
						headers: { 'content-type': 'application/json' },
						body: JSON.stringify({ magnet: t.magnetLink })
					});
					const body = (await res.json()) as
						| {
								streamable: true;
								fileName: string;
								fileSize: number;
								mimeType: string | null;
						  }
						| { streamable: false; reason: string };
					if (body.streamable) {
						result = {
							kind: 'streamable',
							fileName: body.fileName,
							fileSize: body.fileSize,
							mimeType: body.mimeType
						};
					} else {
						result = { kind: 'not-streamable', reason: body.reason };
					}
				} catch (err) {
					const reason = err instanceof Error ? err.message : 'Unknown error';
					result = { kind: 'not-streamable', reason };
				}
				if (runToken !== searchRun) return;
				resultEvals = { ...resultEvals, [t.magnetLink]: result };
			}
		};

		const pool = Math.min(EVAL_CONCURRENCY, matches.length);
		await Promise.all(Array.from({ length: pool }, () => worker()));
	}

	async function startTorrentDownload(magnet: string): Promise<void> {
		const res = await fetch('/api/torrent/add', {
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
	}

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

	function formatDate(value: string): string {
		try {
			return new Date(value).toLocaleString();
		} catch {
			return value;
		}
	}

	function formatBytes(bytes: number): string {
		if (!Number.isFinite(bytes) || bytes <= 0) return '—';
		const units = ['B', 'KB', 'MB', 'GB', 'TB'];
		let value = bytes;
		let unit = 0;
		while (value >= 1024 && unit < units.length - 1) {
			value /= 1024;
			unit++;
		}
		return `${value.toFixed(value >= 10 || unit === 0 ? 0 : 1)} ${units[unit]}`;
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
	<header class="flex flex-wrap items-start justify-between gap-3">
		<div class="flex flex-col gap-1">
			<a class="text-xs text-base-content/60 hover:underline" href="{base}/catalog">← Catalog</a>
			<h1 class="text-2xl font-bold [overflow-wrap:anywhere]">{firkin.title}</h1>
			<p class="text-sm text-base-content/70">
				<span class="badge badge-outline badge-sm">{firkin.addon}</span>
				{#if firkinKind}
					<span class="badge badge-outline badge-sm">{firkinKind}</span>
				{/if}
				{#if firkin.year !== null && firkin.year !== undefined}
					<span class="badge badge-outline badge-sm">{firkin.year}</span>
				{/if}
			</p>
		</div>
		<div class="flex items-center gap-2">
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
			<button
				type="button"
				class="btn gap-2 btn-sm btn-secondary"
				onclick={startIpfsPlay}
				disabled={!hasIpfsFiles || ipfsStarting}
				aria-label="IPFS Play"
				title={hasIpfsFiles
					? 'Stream over IPFS as HLS'
					: 'Available once at least one file is pinned to IPFS'}
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
				<span>{ipfsStarting ? 'Starting…' : 'IPFS Play'}</span>
			</button>
			<button
				type="button"
				class="btn gap-2 btn-sm btn-accent"
				onclick={startTorrentStream}
				disabled={torrentStreamButtonDisabled}
				aria-label="Torrent Stream"
				title={torrentStreamButtonTitle}
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
				<span>{torrentStreamButtonLabel}</span>
			</button>
			<button
				type="button"
				class="btn btn-outline btn-sm btn-error"
				onclick={remove}
				disabled={removing}
			>
				{removing ? 'Deleting…' : 'Delete firkin'}
			</button>
		</div>
	</header>

	{#if removeError}
		<div class="alert alert-error">
			<span>{removeError}</span>
		</div>
	{/if}

	{#if finalizeError}
		<div class="alert alert-error">
			<span>{finalizeError}</span>
		</div>
	{/if}

	{#if ipfsError}
		<div class="alert alert-error">
			<span>{ipfsError}</span>
		</div>
	{/if}

	{#if torrentStreamError}
		<div class="alert alert-error">
			<span>{torrentStreamError}</span>
		</div>
	{/if}

	<div class="grid grid-cols-1 gap-6 lg:grid-cols-[minmax(0,_320px)_1fr]">
		<aside class="flex flex-col gap-4">
			<FirkinCard firkin={firkin as CloudFirkin} />
		</aside>

		<section class="flex flex-col gap-6">
			{#if firkin.description}
				<div class="card border border-base-content/10 bg-base-200 p-4">
					<h2 class="mb-2 text-sm font-semibold text-base-content/70 uppercase">Description</h2>
					<p class="text-sm [overflow-wrap:anywhere] whitespace-pre-wrap">{firkin.description}</p>
				</div>
			{/if}

			<div class="card border border-base-content/10 bg-base-200 p-4">
				<h2 class="mb-2 text-sm font-semibold text-base-content/70 uppercase">Identity</h2>
				<table class="table table-sm">
					<tbody>
						<tr>
							<th class="w-32 align-top">CID</th>
							<td class="font-mono text-xs break-all">{firkin.id}</td>
						</tr>
						<tr>
							<th class="w-32 align-top">Created</th>
							<td class="text-xs">{formatDate(firkin.created_at)}</td>
						</tr>
						<tr>
							<th class="w-32 align-top">Updated</th>
							<td class="text-xs">{formatDate(firkin.updated_at)}</td>
						</tr>
						<tr>
							<th class="w-32 align-top">Version</th>
							<td class="text-xs">{firkin.version ?? 0}</td>
						</tr>
					</tbody>
				</table>
			</div>

			{#if firkin.version_hashes && firkin.version_hashes.length > 0}
				<div class="card border border-base-content/10 bg-base-200 p-4">
					<h2 class="mb-2 text-sm font-semibold text-base-content/70 uppercase">
						Version history ({firkin.version_hashes.length})
					</h2>
					<ol class="list-decimal pl-6 text-xs">
						{#each firkin.version_hashes as cid, i (i)}
							<li class="font-mono break-all">
								<a class="link" href="{base}/catalog/{encodeURIComponent(cid)}">{cid}</a>
							</li>
						{/each}
					</ol>
				</div>
			{/if}

			{#if firkin.artists.length > 0}
				<div class="card border border-base-content/10 bg-base-200 p-4">
					<h2 class="mb-2 text-sm font-semibold text-base-content/70 uppercase">
						Artists ({firkin.artists.length})
					</h2>
					<ul class="flex flex-col gap-3">
						{#each firkin.artists as artist, i (i)}
							<li class="flex items-center gap-3">
								{#if artist.imageUrl}
									<img
										src={cachedImageUrl(artist.imageUrl)}
										alt={artist.name}
										class="h-12 w-12 rounded-full object-cover"
										loading="lazy"
									/>
								{/if}
								<div class="flex flex-col">
									<span class="text-sm font-medium">{artist.name}</span>
									{#if artist.url}
										<a
											class="link text-xs break-all link-primary"
											href={artist.url}
											target="_blank"
											rel="noopener noreferrer">{artist.url}</a
										>
									{/if}
								</div>
							</li>
						{/each}
					</ul>
				</div>
			{/if}

			{#if firkin.images.length > 0}
				<div class="card border border-base-content/10 bg-base-200 p-4">
					<h2 class="mb-2 text-sm font-semibold text-base-content/70 uppercase">
						Images ({firkin.images.length})
					</h2>
					<div class="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4">
						{#each firkin.images as image, i (i)}
							<figure
								class="flex flex-col gap-1 overflow-hidden rounded-box border border-base-content/10 bg-base-300"
							>
								<img
									src={cachedImageUrl(image.url)}
									alt={`Image ${i + 1}`}
									class="block h-auto w-full"
									loading="lazy"
								/>
								<figcaption class="px-2 py-1 text-[10px] text-base-content/70">
									{image.width || '?'}×{image.height || '?'}
									{#if image.fileSize}· {formatBytes(image.fileSize)}{/if}
									{#if image.mimeType}· {image.mimeType}{/if}
								</figcaption>
							</figure>
						{/each}
					</div>
				</div>
			{/if}

			{#if isMusicBrainz}
				<div class="card border border-base-content/10 bg-base-200 p-4">
					<div class="mb-2 flex items-center justify-between gap-2">
						<h2 class="text-sm font-semibold text-base-content/70 uppercase">
							Tracks{tracks.length > 0 ? ` (${tracks.length})` : ''}
						</h2>
					</div>
					{#if tracksStatus === 'loading' && tracks.length === 0}
						<p class="text-sm text-base-content/60">Loading…</p>
					{:else if tracksStatus === 'error'}
						<p class="text-sm text-error">{tracksError ?? 'Failed'}</p>
					{:else if tracks.length === 0}
						<p class="text-sm text-base-content/60">No tracks found.</p>
					{:else}
						{#if trackPlayError}
							<div class="mb-2 alert alert-error">
								<span>{trackPlayError}</span>
							</div>
						{/if}
						<ol class="flex flex-col gap-1">
							{#each tracks as track, idx (track.id || `${track.position}-${track.title}`)}
								{@const playable =
									(track.youtubeStatus === 'found' || track.youtubeStatus === 'idle') &&
									!!track.youtubeUrl}
								{@const isPlaying = playingTrackIndex === idx}
								<li>
									<button
										type="button"
										class={classNames(
											'flex w-full flex-wrap items-center gap-2 rounded border border-base-content/10 px-2 py-1 text-left text-xs',
											{
												'cursor-pointer hover:bg-base-100': playable && !isPlaying,
												'opacity-60': isPlaying,
												'cursor-default': !playable
											}
										)}
										disabled={!playable || playingTrackIndex !== null}
										onclick={() => playTrack(idx)}
										title={playable ? `Play "${track.title}"` : track.title}
									>
										<span class="w-6 shrink-0 text-right font-mono text-base-content/60"
											>{track.position}</span
										>
										<span class="flex-1 truncate">{track.title}</span>
										<span class="text-base-content/60">{formatDuration(track.lengthMs)}</span>
										{#if track.youtubeStatus === 'pending'}
											<span class="badge badge-ghost badge-xs">YT queued</span>
										{:else if track.youtubeStatus === 'searching'}
											<span class="badge badge-ghost badge-xs">YT…</span>
										{:else if playable}
											{#if isPlaying}
												<span class="badge badge-xs badge-primary">starting…</span>
											{:else}
												<span class="badge badge-xs badge-primary">▶ Play</span>
											{/if}
										{:else if track.youtubeStatus === 'missing'}
											<span class="badge badge-xs badge-warning">no match</span>
										{:else if track.youtubeStatus === 'error'}
											<span class="badge badge-xs badge-error">error</span>
										{/if}
									</button>
								</li>
							{/each}
						</ol>
					{/if}
				</div>
			{/if}

			{#if isRetroAchievementsConsole && completedTorrents.length > 0}
				<div class="card border border-base-content/10 bg-base-200 p-4">
					<div class="mb-2 flex items-center justify-between gap-2">
						<h2 class="text-sm font-semibold text-base-content/70 uppercase">
							ROMs{roms && roms.roms.length > 0 ? ` (${roms.roms.length})` : ''}
						</h2>
						<button
							type="button"
							class="btn btn-outline btn-xs"
							onclick={() => loadRoms(true)}
							disabled={romsStatus === 'loading'}
						>
							{romsStatus === 'loading' ? 'Scanning…' : 'Rescan'}
						</button>
					</div>
					{#if romsStatus === 'loading' && !roms}
						<p class="text-sm text-base-content/60">Extracting archives and scanning for ROMs…</p>
					{:else if romsStatus === 'error'}
						<p class="text-sm text-error">{romsError ?? 'Failed'}</p>
					{:else if roms}
						{#if roms.archives.length > 0}
							<div class="mb-3 flex flex-col gap-1">
								<h3 class="text-xs font-semibold text-base-content/60 uppercase">
									Archives ({roms.archives.length})
								</h3>
								<ul class="flex flex-col gap-1">
									{#each roms.archives as archive (archive.relative_path)}
										<li class="flex flex-wrap items-center gap-2 text-xs">
											<span
												class={classNames('badge badge-sm', {
													'badge-success': archive.status === 'extracted',
													'badge-ghost':
														archive.status === 'already_extracted' || archive.status === 'skipped',
													'badge-error': archive.status === 'failed'
												})}
											>
												{archive.status === 'already_extracted' ? 'extracted' : archive.status}
											</span>
											<span class="font-mono [overflow-wrap:anywhere]">{archive.relative_path}</span
											>
											{#if archive.error}
												<span class="text-error">— {archive.error}</span>
											{/if}
										</li>
									{/each}
								</ul>
							</div>
						{/if}
						{#if roms.roms.length === 0}
							<p class="text-sm text-base-content/60">
								No ROM files found yet. If the torrent contains a compressed archive, extraction may
								still be in progress — try Rescan.
							</p>
						{:else}
							<div class="overflow-x-auto rounded-box border border-base-content/10">
								<table class="table table-sm">
									<thead>
										<tr>
											<th>File</th>
											<th>Path</th>
											<th class="text-right">Size</th>
										</tr>
									</thead>
									<tbody>
										{#each roms.roms as rom (rom.relative_path)}
											<tr>
												<td class="text-xs [overflow-wrap:anywhere]">{rom.name}</td>
												<td class="font-mono text-xs [overflow-wrap:anywhere]"
													>{rom.relative_path}</td
												>
												<td class="text-right text-xs">{formatBytes(rom.size)}</td>
											</tr>
										{/each}
									</tbody>
								</table>
							</div>
						{/if}
					{/if}
				</div>
			{/if}

			{#if hasMagnetFiles}
				<div class="card border border-base-content/10 bg-base-200 p-4">
					<div class="flex items-center justify-between gap-2">
						<button
							type="button"
							class="flex flex-1 items-center gap-2 text-left"
							onclick={toggleTorrentSearch}
							aria-expanded={torrentSearchOpen}
						>
							<span class="text-base-content/60" aria-hidden="true"
								>{torrentSearchOpen ? '▼' : '▶'}</span
							>
							<h2 class="text-sm font-semibold text-base-content/70 uppercase">
								Torrent search{torrentSearchOpen && torrentMatches.length > 0
									? ` (${torrentMatches.length})`
									: ''}
							</h2>
						</button>
						{#if torrentSearchOpen}
							<button
								type="button"
								class="btn btn-outline btn-xs"
								onclick={() => runTorrentSearch(firkin.id, firkin.title, firkin.addon, firkin.year)}
								disabled={torrentStatus === 'searching'}
							>
								{torrentStatus === 'searching' ? 'Searching…' : 'Refresh'}
							</button>
						{/if}
					</div>
					{#if torrentSearchOpen}
						<div class="mt-2">
							{#if assignError}
								<div class="mb-2 alert alert-error">
									<span>{assignError}</span>
								</div>
							{/if}
							{#if torrentStatus === 'searching' && torrentMatches.length === 0}
								<p class="text-sm text-base-content/60">Searching…</p>
							{:else if torrentStatus === 'error'}
								<p class="text-sm text-error">{torrentError ?? 'Failed'}</p>
							{:else if torrentMatches.length === 0}
								<p class="text-sm text-base-content/60">No matching torrents.</p>
							{:else}
								<div class="flex flex-col gap-1">
									{#each torrentMatches as torrent (torrent.infoHash)}
										{@const added = existingHashes.has(torrent.magnetLink)}
										{@const adding = addingHash === torrent.magnetLink}
										{@const streamEvalRow: TorrentRowEval = torrent.magnetLink
									? (resultEvals[torrent.magnetLink] ?? ({ kind: 'pending' } as TorrentRowEval))
									: ({ kind: 'not-streamable', reason: 'no magnet' } as TorrentRowEval)}
										<button
											type="button"
											class={classNames(
												'flex flex-wrap items-center gap-2 rounded border border-base-content/10 px-2 py-1 text-left text-xs hover:bg-base-100',
												{ 'opacity-60': added || adding }
											)}
											onclick={() => assignTorrent(torrent)}
											disabled={addingHash !== null || added}
											title={streamEvalRow.kind === 'streamable'
												? `Streamable — ${streamEvalRow.fileName} · ${torrent.title}`
												: streamEvalRow.kind === 'not-streamable'
													? `Not streamable: ${streamEvalRow.reason} · ${torrent.title}`
													: torrent.title}
										>
											{#if streamEvalRow.kind === 'pending' || streamEvalRow.kind === 'evaluating'}
												<span
													class="loading loading-xs shrink-0 loading-spinner text-base-content/50"
													aria-label="Probing torrent metadata"
												></span>
											{:else if streamEvalRow.kind === 'streamable'}
												<span
													class="shrink-0 text-success"
													aria-label="Streamable"
													title={`Streamable — ${streamEvalRow.fileName}`}
												>
													<svg
														xmlns="http://www.w3.org/2000/svg"
														viewBox="0 0 24 24"
														fill="currentColor"
														class="h-3.5 w-3.5"
														aria-hidden="true"
													>
														<polygon points="6 4 20 12 6 20 6 4" />
													</svg>
												</span>
											{:else}
												<span
													class="shrink-0 text-base-content/30"
													aria-label="Not streamable"
													title={`Not streamable: ${streamEvalRow.reason}`}
												>
													<svg
														xmlns="http://www.w3.org/2000/svg"
														viewBox="0 0 24 24"
														fill="none"
														stroke="currentColor"
														stroke-width="2.5"
														stroke-linecap="round"
														class="h-3.5 w-3.5"
														aria-hidden="true"
													>
														<line x1="5" y1="5" x2="19" y2="19" />
														<line x1="19" y1="5" x2="5" y2="19" />
													</svg>
												</span>
											{/if}
											<span class="font-medium">{torrent.quality ?? '—'}</span>
											<span class="text-success">↑{torrent.seeders}</span>
											<span class="text-warning">↓{torrent.leechers}</span>
											<span class="text-base-content/60">{formatSizeBytes(torrent.sizeBytes)}</span>
											<span class="truncate text-base-content/70"
												>{torrent.parsedTitle || torrent.title}</span
											>
											{#if added}
												<span class="ml-auto">✓</span>
											{:else if adding}
												<span class="ml-auto">…</span>
											{/if}
										</button>
									{/each}
								</div>
							{/if}
						</div>
					{/if}
				</div>
			{/if}

			<div class="card border border-base-content/10 bg-base-200 p-4">
				<h2 class="mb-2 text-sm font-semibold text-base-content/70 uppercase">
					Files ({firkin.files.length})
				</h2>
				{#if firkin.files.length === 0}
					<p class="text-sm text-base-content/60">No files attached.</p>
				{:else}
					<div class="overflow-x-auto rounded-box border border-base-content/10">
						<table class="table table-sm">
							<thead>
								<tr>
									<th class="w-24">Type</th>
									<th>Title</th>
									<th>Value</th>
								</tr>
							</thead>
							<tbody>
								{#each firkin.files as file, i (i)}
									<tr>
										<td class={classNames('text-xs font-semibold')}>
											<span class="badge badge-outline badge-sm">{file.type}</span>
										</td>
										<td class="text-xs [overflow-wrap:anywhere]">{file.title ?? ''}</td>
										<td class="font-mono text-xs break-all">{file.value}</td>
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				{/if}
			</div>
		</section>
	</div>
</div>
