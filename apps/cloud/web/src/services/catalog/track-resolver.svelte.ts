import { base } from '$app/paths';
import { playYouTubeAudio, resolveYouTubeUrlForTrack } from '$lib/youtube-match.service';
import { resolveLyricsForTrack } from '$lib/lrclib-match.service';
import type { FileEntry } from '$lib/firkins.service';
import type { ResolutionStatus, TrackEntry } from '$services/catalog/types';
import type { SubsLyricsItem } from '$types/subs-lyrics.type';

export interface TrackResolverOptions {
	persistTrackUrls?: (resolved: { title: string; url: string }[]) => Promise<void>;
}

interface LoadArgs {
	releaseGroupId: string;
	savedUrls?: Record<string, string>;
}

interface ResolveArgs {
	albumTitle: string;
	artist: string;
	thumb: string | null;
}

export class TrackResolver {
	tracks = $state<TrackEntry[]>([]);
	status = $state<ResolutionStatus>('idle');
	error = $state<string | null>(null);
	playingIndex = $state<number | null>(null);
	playError = $state<string | null>(null);

	private run = 0;
	private readonly persistTrackUrls?: (resolved: { title: string; url: string }[]) => Promise<void>;

	constructor(options: TrackResolverOptions = {}) {
		this.persistTrackUrls = options.persistTrackUrls;
	}

	cancel(): void {
		this.run++;
	}

	seedFromFiles(files: FileEntry[]): void {
		const myRun = ++this.run;
		const trackFiles = files.filter((f) => f.type === 'url' && (f.title ?? '').trim().length > 0);
		this.tracks = trackFiles.map((f, i) => ({
			id: `file-${i}`,
			position: i + 1,
			title: f.title ?? '',
			lengthMs: null,
			youtubeUrl: f.value || null,
			youtubeStatus: f.value ? 'idle' : 'pending',
			lyricsStatus: 'pending',
			lyrics: null
		}));
		this.status = 'done';
		this.error = null;
		// `myRun` keeps follow-up resolves on this resolver from clobbering
		// the seeded list.
		void myRun;
	}

	async loadByReleaseGroup(args: LoadArgs, resolveArgs: ResolveArgs): Promise<void> {
		const myRun = ++this.run;
		this.status = 'loading';
		this.error = null;
		this.tracks = [];

		try {
			const body = await fetchMusicBrainzTracks(args.releaseGroupId);
			if (myRun !== this.run) return;
			this.tracks = body.map((t) => {
				const savedUrl = args.savedUrls?.[t.title.trim().toLowerCase()] ?? null;
				return {
					id: t.id,
					position: t.position,
					title: t.title,
					lengthMs: t.lengthMs,
					youtubeUrl: savedUrl,
					youtubeStatus: (savedUrl ? 'idle' : 'pending') as TrackEntry['youtubeStatus'],
					lyricsStatus: 'pending' as TrackEntry['lyricsStatus'],
					lyrics: null
				};
			});
			this.status = 'done';
			await Promise.all([
				this.resolveAllYouTube(myRun, resolveArgs),
				this.resolveAllLyrics(myRun, resolveArgs)
			]);
		} catch (err) {
			if (myRun !== this.run) return;
			this.error = err instanceof Error ? err.message : 'Unknown error';
			this.status = 'error';
		}
	}

	async resolveAllForCurrent(resolveArgs: ResolveArgs): Promise<void> {
		const myRun = ++this.run;
		await Promise.all([
			this.resolveAllYouTube(myRun, resolveArgs),
			this.resolveAllLyrics(myRun, resolveArgs)
		]);
	}

	resolvedTrackFiles(): FileEntry[] {
		return this.tracks
			.filter(
				(t): t is TrackEntry & { youtubeUrl: string } =>
					(t.youtubeStatus === 'found' || t.youtubeStatus === 'idle') &&
					Boolean(t.youtubeUrl) &&
					t.title.trim().length > 0
			)
			.map((t) => ({ type: 'url', value: t.youtubeUrl, title: t.title }));
	}

	async play(index: number, opts: { thumb: string | null }): Promise<void> {
		const t = this.tracks[index];
		if (!t || !t.youtubeUrl || this.playingIndex !== null) return;
		this.playingIndex = index;
		this.playError = null;
		try {
			const durationSeconds = t.lengthMs ? Math.round(t.lengthMs / 1000) : null;
			await playYouTubeAudio(t.youtubeUrl, t.title, opts.thumb, durationSeconds);
		} catch (err) {
			this.playError = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			this.playingIndex = null;
		}
	}

	private async resolveAllLyrics(myRun: number, args: ResolveArgs): Promise<void> {
		for (let i = 0; i < this.tracks.length; i++) {
			if (myRun !== this.run) return;
			const t = this.tracks[i];
			if (t.lyricsStatus !== 'pending') continue;
			this.tracks = this.tracks.map((tr, idx) =>
				idx === i ? { ...tr, lyricsStatus: 'searching' } : tr
			);
			let lyrics: SubsLyricsItem | null = null;
			try {
				lyrics = await resolveLyricsForTrack(t.title, args.artist, args.albumTitle, t.lengthMs);
				if (myRun !== this.run) return;
				this.tracks = this.tracks.map((tr, idx) =>
					idx === i ? { ...tr, lyrics, lyricsStatus: lyrics ? 'found' : 'missing' } : tr
				);
			} catch {
				if (myRun !== this.run) return;
				this.tracks = this.tracks.map((tr, idx) =>
					idx === i ? { ...tr, lyrics: null, lyricsStatus: 'error' } : tr
				);
			}
		}
	}

	private async resolveAllYouTube(myRun: number, args: ResolveArgs): Promise<void> {
		const newlyResolved: { title: string; url: string }[] = [];
		for (let i = 0; i < this.tracks.length; i++) {
			if (myRun !== this.run) return;
			const t = this.tracks[i];
			if (t.youtubeStatus === 'idle' || t.youtubeStatus === 'found') continue;
			this.tracks = this.tracks.map((tr, idx) =>
				idx === i ? { ...tr, youtubeStatus: 'searching' } : tr
			);
			let url: string | null = null;
			try {
				url = await resolveYouTubeUrlForTrack(t.title, args.artist, args.albumTitle, t.lengthMs);
				if (myRun !== this.run) return;
				this.tracks = this.tracks.map((tr, idx) =>
					idx === i ? { ...tr, youtubeUrl: url, youtubeStatus: url ? 'found' : 'missing' } : tr
				);
			} catch {
				if (myRun !== this.run) return;
				this.tracks = this.tracks.map((tr, idx) =>
					idx === i ? { ...tr, youtubeUrl: null, youtubeStatus: 'error' } : tr
				);
				continue;
			}
			if (url) newlyResolved.push({ title: t.title, url });
		}
		if (myRun !== this.run || newlyResolved.length === 0) return;
		if (this.persistTrackUrls) {
			try {
				await this.persistTrackUrls(newlyResolved);
			} catch (err) {
				console.warn('[track-resolver] persist failed', err);
			}
		}
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
