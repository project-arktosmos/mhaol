import { base } from '$app/paths';
import { playYouTubeAudio } from '$lib/youtube-match.service';
import type { FileEntry } from '$lib/firkins.service';
import { playerService } from '$services/player.service';
import type { ResolutionStatus, TrackEntry } from '$services/catalog/types';
import type { SubsLyricsItem, SubsLyricsSyncedLine } from '$types/subs-lyrics.type';
import type { PlaylistTrack } from '$types/player.type';

interface LoadFromFirkinArgs {
	releaseGroupId: string;
	files: FileEntry[];
}

export interface AlbumProgressLyrics {
	source: string;
	externalId: string;
	syncedLyrics?: string;
	plainLyrics?: string;
	instrumental: boolean;
}

export interface AlbumProgressTrack {
	position: number;
	title: string;
	lengthMs?: number | null;
	youtubeStatus: 'pending' | 'searching' | 'found' | 'missing' | 'error';
	youtubeUrl?: string | null;
	lyricsStatus: 'pending' | 'searching' | 'found' | 'missing' | 'error';
	lyrics?: AlbumProgressLyrics | null;
}

export interface AlbumProgressPayload {
	firkinId: string;
	started_at: string;
	updated_at: string;
	completed: boolean;
	completedId?: string | null;
	tracks: AlbumProgressTrack[];
}

function mapEntryStatus(
	s: AlbumProgressTrack['youtubeStatus']
): 'pending' | 'searching' | 'found' | 'missing' | 'error' {
	return s;
}

function decodeLyricsProgress(p: AlbumProgressLyrics, title: string): SubsLyricsItem {
	const synced = p.syncedLyrics ? parseLrcText(p.syncedLyrics) : undefined;
	return {
		kind: 'lyrics',
		source: p.source,
		externalId: p.externalId,
		trackName: title,
		plainLyrics: p.plainLyrics,
		syncedLyrics: synced && synced.length > 0 ? synced : undefined,
		instrumental: p.instrumental === true,
		format: synced && synced.length > 0 ? 'lrc' : undefined
	};
}

/// Pure projection over a MusicBrainz album. Track YouTube URLs and
/// lyrics live on the firkin's `files` (resolved server-side by
/// `POST /api/firkins/:id/resolve-tracks`, also auto-spawned as a
/// background task by `POST /api/firkins` for fresh musicbrainz
/// albums). This class doesn't search anything itself; it just fetches
/// the MB tracklist and pairs each track with its persisted
/// (or persisted-empty) entries. The caller (detail page) polls the
/// firkin while `missingAny === true` and re-projects when the
/// background task rolls the firkin forward.
export class TrackResolver {
	tracks = $state<TrackEntry[]>([]);
	status = $state<ResolutionStatus>('idle');
	error = $state<string | null>(null);
	playingIndex = $state<number | null>(null);
	playError = $state<string | null>(null);

	private run = 0;

	cancel(): void {
		this.run++;
	}

	async loadFromFirkin(args: LoadFromFirkinArgs): Promise<{ missingAny: boolean }> {
		const myRun = ++this.run;
		this.status = 'loading';
		this.error = null;
		this.tracks = [];

		try {
			const tracks = await fetchMusicBrainzTracks(args.releaseGroupId);
			if (myRun !== this.run) return { missingAny: false };
			const ytByTitle = new Map<string, string>();
			const lyricsByTitle = new Map<string, SubsLyricsItem>();
			for (const f of args.files) {
				const title = (f.title ?? '').trim().toLowerCase();
				if (!title) continue;
				if (f.type === 'url' && isYouTubeUrl(f.value)) {
					ytByTitle.set(title, f.value);
				} else if (f.type === 'lyrics' && f.value) {
					const item = decodeLyricsValue(f.value, f.title ?? '');
					if (item) lyricsByTitle.set(title, item);
				}
			}

			let missingAny = false;
			this.tracks = tracks.map((t) => {
				const key = t.title.trim().toLowerCase();
				const youtubeUrl = ytByTitle.get(key) ?? null;
				const lyrics = lyricsByTitle.get(key) ?? null;
				if (!youtubeUrl) missingAny = true;
				if (!lyrics) missingAny = true;
				return {
					id: t.id,
					position: t.position,
					title: t.title,
					lengthMs: t.lengthMs,
					youtubeUrl,
					youtubeStatus: youtubeUrl ? 'idle' : 'pending',
					lyrics,
					lyricsStatus: lyrics ? 'found' : 'pending'
				};
			});
			this.status = 'done';
			return { missingAny };
		} catch (err) {
			if (myRun !== this.run) return { missingAny: false };
			this.error = err instanceof Error ? err.message : 'Unknown error';
			this.status = 'error';
			return { missingAny: false };
		}
	}

	/// Overlay the server-side resolution progress map onto the
	/// projected tracks. Called by the detail page each time it polls
	/// `/api/firkins/:id/resolution-progress` while the background
	/// album resolver is in flight, so per-track YT URL / lyrics status
	/// updates render in real time without waiting for the firkin to
	/// roll forward.
	///
	/// Idempotent: returns without touching `this.tracks` when every
	/// track's projected fields are already in their final state. Each
	/// reactive write to `tracks` propagates through every consumer
	/// (CatalogTracksCard, derived stats, …); writing on every poll tick
	/// regardless of whether anything changed used to fight the player's
	/// own state writes inside Svelte's effect-flush loop, occasionally
	/// tripping the runtime's update-depth guard.
	applyProgress(progress: AlbumProgressPayload): void {
		const byTitle = new Map<string, AlbumProgressTrack>();
		for (const t of progress.tracks) {
			const key = t.title.trim().toLowerCase();
			if (key) byTitle.set(key, t);
		}
		let changed = false;
		const next: TrackEntry[] = new Array(this.tracks.length);
		for (let i = 0; i < this.tracks.length; i++) {
			const tr = this.tracks[i];
			const key = tr.title.trim().toLowerCase();
			const p = byTitle.get(key);
			if (!p) {
				next[i] = tr;
				continue;
			}
			const ytStatus = mapEntryStatus(p.youtubeStatus);
			const lyStatus = mapEntryStatus(p.lyricsStatus);
			const newYtUrl = tr.youtubeUrl ?? p.youtubeUrl ?? null;
			const newYtStatus = newYtUrl ? 'idle' : ytStatus === 'found' ? 'found' : ytStatus;
			let newLyrics = tr.lyrics;
			if (!newLyrics && p.lyrics) {
				newLyrics = decodeLyricsProgress(p.lyrics, tr.title);
			}
			const newLyStatus = newLyrics ? 'found' : lyStatus;

			if (
				tr.youtubeUrl === newYtUrl &&
				tr.youtubeStatus === newYtStatus &&
				tr.lyrics === newLyrics &&
				tr.lyricsStatus === newLyStatus
			) {
				next[i] = tr;
				continue;
			}
			changed = true;
			next[i] = {
				...tr,
				youtubeUrl: newYtUrl,
				youtubeStatus: newYtStatus,
				lyrics: newLyrics,
				lyricsStatus: newLyStatus
			};
		}
		if (changed) this.tracks = next;
	}

	async play(
		index: number,
		opts: { thumb: string | null; albumTitle?: string }
	): Promise<void> {
		const t = this.tracks[index];
		if (!t || !t.youtubeUrl || this.playingIndex !== null) return;
		this.playingIndex = index;
		this.playError = null;
		try {
			const durationSeconds = t.lengthMs ? Math.round(t.lengthMs / 1000) : null;
			const syncedLyrics =
				t.lyrics?.syncedLyrics && t.lyrics.syncedLyrics.length > 0
					? t.lyrics.syncedLyrics
					: null;
			// Surface the full tracklist to the floating player panel so the
			// user can swap between songs from anywhere in the app — even
			// after navigating away from this catalog page (the playlist
			// lives on `playerService` and persists across track swaps).
			const playlistTracks: PlaylistTrack[] = this.tracks.map((tr) => ({
				title: tr.title,
				youtubeUrl: tr.youtubeUrl ?? null,
				thumbnailUrl: opts.thumb,
				durationSeconds: tr.lengthMs ? Math.round(tr.lengthMs / 1000) : null,
				syncedLyrics:
					tr.lyrics?.syncedLyrics && tr.lyrics.syncedLyrics.length > 0
						? tr.lyrics.syncedLyrics
						: null,
				position: tr.position
			}));
			playerService.setPlaylist({
				tracks: playlistTracks,
				currentIndex: index,
				title: opts.albumTitle
			});
			await playYouTubeAudio(t.youtubeUrl, t.title, opts.thumb, durationSeconds, syncedLyrics);
		} catch (err) {
			this.playError = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			this.playingIndex = null;
		}
	}
}

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

function parseLrcText(lrc: string): SubsLyricsSyncedLine[] {
	const lines: SubsLyricsSyncedLine[] = [];
	for (const raw of lrc.split('\n')) {
		const line = raw.trim();
		if (!line.startsWith('[')) continue;
		const close = line.indexOf(']');
		if (close < 0) continue;
		const ts = line.slice(1, close);
		const text = line.slice(close + 1).trim();
		const m = ts.match(/^(\d+):(\d+(?:\.\d+)?)$/);
		if (!m) continue;
		const minutes = Number.parseFloat(m[1]);
		const seconds = Number.parseFloat(m[2]);
		if (!Number.isFinite(minutes) || !Number.isFinite(seconds)) continue;
		lines.push({ time: minutes * 60 + seconds, text });
	}
	lines.sort((a, b) => a.time - b.time);
	return lines;
}

/// Decode the JSON blob the server stores under a `lyrics`-typed file
/// entry (`{source, externalId, syncedLyrics, plainLyrics, instrumental}`)
/// into a `SubsLyricsItem` the existing display components understand.
function decodeLyricsValue(value: string, title: string): SubsLyricsItem | null {
	try {
		const parsed = JSON.parse(value) as {
			source?: string;
			externalId?: string;
			syncedLyrics?: string | null;
			plainLyrics?: string | null;
			instrumental?: boolean;
		};
		const synced = parsed.syncedLyrics ? parseLrcText(parsed.syncedLyrics) : undefined;
		return {
			kind: 'lyrics',
			source: parsed.source ?? 'lrclib',
			externalId: parsed.externalId ?? '',
			trackName: title,
			plainLyrics: parsed.plainLyrics ?? undefined,
			syncedLyrics: synced && synced.length > 0 ? synced : undefined,
			instrumental: parsed.instrumental === true,
			format: synced && synced.length > 0 ? 'lrc' : undefined
		};
	} catch {
		return null;
	}
}

async function fetchMusicBrainzTracks(releaseGroupId: string): Promise<
	{
		id: string;
		position: number;
		title: string;
		lengthMs: number | null;
	}[]
> {
	const res = await fetch(
		`${base}/api/catalog/musicbrainz/release-groups/${encodeURIComponent(releaseGroupId)}/tracks`,
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
	return (await res.json()) as {
		id: string;
		position: number;
		title: string;
		lengthMs: number | null;
	}[];
}
