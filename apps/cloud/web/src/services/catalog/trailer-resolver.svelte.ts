import { base } from '$app/paths';
import {
	playYouTubeVideo,
	resolveYouTubeTrailerForMovie,
	resolveYouTubeTrailerForSeason
} from '$lib/youtube-match.service';
import type { Trailer } from '$lib/firkins.service';
import type { ResolutionStatus, TrailerEntry } from '$services/catalog/types';

export interface TrailerResolverOptions {
	persist?: (resolved: Trailer[]) => Promise<void>;
	/**
	 * When set, the resolver auto-plays the first playable trailer through
	 * the right-side player once per resolution run. The callback supplies
	 * the playback context (firkin title + thumb); returning `null` skips
	 * auto-play for this run.
	 */
	autoPlay?: () => { firkinTitle: string; thumb: string | null } | null;
}

interface MovieArgs {
	addon: string;
	tmdbMovieId: string | null;
	title: string;
	year: number | null;
	stored?: Trailer[];
}

interface TvArgs {
	addon: string;
	tmdbTvId: string | null;
	title: string;
	stored?: Trailer[];
}

export class TrailerResolver {
	trailers = $state<TrailerEntry[]>([]);
	status = $state<ResolutionStatus>('idle');
	error = $state<string | null>(null);
	playingKey = $state<string | null>(null);
	playError = $state<string | null>(null);

	private run = 0;
	private autoPlayedRun = -1;
	private readonly persist?: (resolved: Trailer[]) => Promise<void>;
	private readonly autoPlayContext?: () => {
		firkinTitle: string;
		thumb: string | null;
	} | null;

	constructor(options: TrailerResolverOptions = {}) {
		this.persist = options.persist;
		this.autoPlayContext = options.autoPlay;
	}

	cancel(): void {
		this.run++;
	}

	async resolveMovie(args: MovieArgs): Promise<void> {
		const myRun = ++this.run;
		this.status = 'loading';
		this.error = null;
		this.playError = null;

		const stored = args.stored ?? [];
		const existing = stored.find((t) => Boolean(t.youtubeUrl));
		if (existing) {
			this.trailers = [
				{
					key: 'movie',
					label: existing.label ?? null,
					seasonNumber: null,
					airYear: args.year,
					youtubeUrl: existing.youtubeUrl,
					language: existing.language ?? null,
					status: 'idle'
				}
			];
			this.status = 'done';
			this.maybeAutoPlay(myRun);
			return;
		}

		if (args.tmdbMovieId) {
			try {
				const tmdb = await fetchTrailers(args.addon, args.tmdbMovieId);
				if (myRun !== this.run) return;
				const first = tmdb[0];
				if (first) {
					this.trailers = [
						{
							key: 'movie',
							label: first.label ?? null,
							seasonNumber: null,
							airYear: args.year,
							youtubeUrl: first.youtubeUrl,
							language: first.language ?? null,
							status: 'idle'
						}
					];
					this.status = 'done';
					await this.maybePersist([first]);
					this.maybeAutoPlay(myRun);
					return;
				}
			} catch (err) {
				console.warn('[trailer-resolver] tmdb metadata fetch failed', err);
			}
		}

		this.trailers = [
			{
				key: 'movie',
				label: null,
				seasonNumber: null,
				airYear: args.year,
				youtubeUrl: null,
				language: null,
				status: 'searching'
			}
		];
		try {
			const url = await resolveYouTubeTrailerForMovie(args.title, args.year);
			if (myRun !== this.run) return;
			this.trailers = this.trailers.map((t) =>
				t.key === 'movie' ? { ...t, youtubeUrl: url, status: url ? 'found' : 'missing' } : t
			);
			this.status = 'done';
			if (url) {
				await this.maybePersist([{ youtubeUrl: url }]);
				this.maybeAutoPlay(myRun);
			}
		} catch (err) {
			if (myRun !== this.run) return;
			this.trailers = this.trailers.map((t) => (t.key === 'movie' ? { ...t, status: 'error' } : t));
			this.error = err instanceof Error ? err.message : 'Unknown error';
			this.status = 'error';
		}
	}

	async resolveTv(args: TvArgs): Promise<void> {
		const myRun = ++this.run;
		this.status = 'loading';
		this.error = null;
		this.playError = null;

		const stored = args.stored ?? [];
		if (stored.length > 0) {
			this.trailers = stored.map((t, i) => ({
				key: t.label ? `season-${t.label}` : `trailer-${i}`,
				label: t.label ?? null,
				seasonNumber: parseSeasonNumberFromLabel(t.label ?? ''),
				airYear: null,
				youtubeUrl: t.youtubeUrl,
				language: t.language ?? null,
				status: 'idle' as const
			}));
			this.status = 'done';
			this.maybeAutoPlay(myRun);
			return;
		}

		if (!args.tmdbTvId) {
			this.error = 'No TMDB id stored on this firkin. Re-bookmark from the catalog to attach one.';
			this.status = 'error';
			return;
		}

		let tmdbShowTrailers: Trailer[] = [];
		try {
			tmdbShowTrailers = await fetchTrailers(args.addon, args.tmdbTvId);
			if (myRun !== this.run) return;
		} catch (err) {
			console.warn('[trailer-resolver] tmdb tv metadata fetch failed', err);
		}

		let seasons: { seasonNumber: number; name: string; airYear: number | null }[];
		try {
			seasons = await fetchTvSeasons(args.tmdbTvId);
			if (myRun !== this.run) return;
		} catch (err) {
			if (myRun !== this.run) return;
			this.error = err instanceof Error ? err.message : 'Unknown error';
			this.status = 'error';
			return;
		}

		const tmdbEntries: TrailerEntry[] = tmdbShowTrailers.map((t, i) => ({
			key: `tmdb-${i}`,
			label: t.label ?? 'Trailer',
			seasonNumber: null,
			airYear: null,
			youtubeUrl: t.youtubeUrl,
			language: t.language ?? null,
			status: 'idle' as const
		}));
		const seasonEntries: TrailerEntry[] = seasons.map((s) => ({
			key: `season-${s.seasonNumber}`,
			label: s.name,
			seasonNumber: s.seasonNumber,
			airYear: s.airYear,
			youtubeUrl: null,
			language: null,
			status: 'pending' as const
		}));
		this.trailers = [...tmdbEntries, ...seasonEntries];
		this.status = 'done';
		this.maybeAutoPlay(myRun);

		const resolved: Trailer[] = tmdbShowTrailers
			.filter((t): t is Trailer & { youtubeUrl: string } => Boolean(t.youtubeUrl))
			.map((t) => ({ youtubeUrl: t.youtubeUrl, label: t.label, language: t.language }));

		for (let i = 0; i < this.trailers.length; i++) {
			if (myRun !== this.run) return;
			const entry = this.trailers[i];
			if (entry.status === 'idle') continue;
			this.trailers = this.trailers.map((t, idx) =>
				idx === i ? { ...t, status: 'searching' } : t
			);
			try {
				const url = await resolveYouTubeTrailerForSeason(
					args.title,
					entry.seasonNumber ?? 0,
					entry.airYear
				);
				if (myRun !== this.run) return;
				this.trailers = this.trailers.map((t, idx) =>
					idx === i ? { ...t, youtubeUrl: url, status: url ? 'found' : 'missing' } : t
				);
				if (url && entry.label) {
					resolved.push({ youtubeUrl: url, label: entry.label });
				}
				this.maybeAutoPlay(myRun);
			} catch {
				if (myRun !== this.run) return;
				this.trailers = this.trailers.map((t, idx) => (idx === i ? { ...t, status: 'error' } : t));
			}
		}

		if (resolved.length > 0) await this.maybePersist(resolved);
	}

	resolvedTrailers(): Trailer[] {
		return this.trailers
			.filter((t): t is TrailerEntry & { youtubeUrl: string } => Boolean(t.youtubeUrl))
			.map((t) => ({
				youtubeUrl: t.youtubeUrl,
				label: t.label ?? undefined,
				language: t.language ?? undefined
			}));
	}

	async play(
		entry: TrailerEntry,
		opts: { firkinTitle: string; thumb: string | null; autoplay?: boolean }
	): Promise<void> {
		if (!entry.youtubeUrl || this.playingKey !== null) return;
		this.playingKey = entry.key;
		this.playError = null;
		try {
			const playTitle = entry.label
				? `${opts.firkinTitle} — ${entry.label} trailer`
				: `${opts.firkinTitle} trailer`;
			await playYouTubeVideo(entry.youtubeUrl, playTitle, opts.thumb, null, {
				autoplay: opts.autoplay !== false
			});
		} catch (err) {
			this.playError = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			this.playingKey = null;
		}
	}

	private async maybePersist(resolved: Trailer[]): Promise<void> {
		if (!this.persist) return;
		try {
			await this.persist(resolved);
		} catch (err) {
			console.warn('[trailer-resolver] persist failed', err);
		}
	}

	private maybeAutoPlay(myRun: number): void {
		if (!this.autoPlayContext) return;
		if (myRun !== this.run) return;
		if (this.autoPlayedRun === myRun) return;
		if (this.playingKey !== null) return;
		const first = this.trailers.find((t) => Boolean(t.youtubeUrl));
		if (!first) return;
		const ctx = this.autoPlayContext();
		if (!ctx) return;
		this.autoPlayedRun = myRun;
		// Load the trailer into the right-side player but defer the actual
		// playback — `PlayerVideo` renders a big centered play overlay
		// while `awaitingPlay` is set so the user can press play.
		void this.play(first, { ...ctx, autoplay: false });
	}
}

async function fetchTrailers(addon: string, id: string): Promise<Trailer[]> {
	const res = await fetch(
		`${base}/api/catalog/${encodeURIComponent(addon)}/${encodeURIComponent(id)}/metadata`,
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
	const body = (await res.json()) as { trailers?: Trailer[] };
	return Array.isArray(body.trailers) ? body.trailers : [];
}

async function fetchTvSeasons(
	tmdbTvId: string
): Promise<{ seasonNumber: number; name: string; airYear: number | null }[]> {
	const res = await fetch(`${base}/api/catalog/tmdb-tv/${encodeURIComponent(tmdbTvId)}/seasons`, {
		cache: 'no-store'
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
		seasonNumber: number;
		name: string;
		airYear?: number | null;
	}[];
	return body.map((s) => ({
		seasonNumber: s.seasonNumber,
		name: s.name,
		airYear: s.airYear ?? null
	}));
}

function parseSeasonNumberFromLabel(label: string): number | null {
	const m = label.match(/season\s+(\d+)/i);
	if (!m) return null;
	const n = Number.parseInt(m[1], 10);
	return Number.isFinite(n) ? n : null;
}
