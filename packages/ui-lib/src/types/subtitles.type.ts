import type { WyzieSubtitle } from 'addons/wyzie-subs/types';

export type SubtitleMediaType = 'movie' | 'tv';

export interface SubtitleSearchContext {
	type: SubtitleMediaType;
	tmdbId: string;
	imdbId?: string;
	season?: number;
	episode?: number;
}

export type SubtitleSearchResult = WyzieSubtitle;

export interface AssignedSubtitle {
	id: string;
	mediaKey: string;
	languageCode: string;
	languageName: string;
	source: string;
	sourceId: string | null;
	format: string;
	hearingImpaired: boolean;
	url: string;
	downloadedAt: string;
}

/// Build a stable key identifying which "thing" a subtitle is attached to.
/// movies → `movie:{tmdbId}`, TV episodes → `tv:{tmdbId}:s{nn}e{nn}`.
export function buildMediaKey(ctx: SubtitleSearchContext): string {
	if (ctx.type === 'tv') {
		const s = ctx.season ?? 0;
		const e = ctx.episode ?? 0;
		const sp = String(s).padStart(2, '0');
		const ep = String(e).padStart(2, '0');
		return `tv:${ctx.tmdbId}:s${sp}e${ep}`;
	}
	return `movie:${ctx.tmdbId}`;
}
