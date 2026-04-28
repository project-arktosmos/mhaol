export type WyzieMediaType = 'movie' | 'tv';

export interface WyzieSearchParams {
	type: WyzieMediaType;
	tmdbId?: string;
	imdbId?: string;
	season?: number;
	episode?: number;
	languages?: string[];
	hearingImpaired?: boolean;
}

export interface WyzieSubtitle {
	id: string;
	url: string;
	format: string;
	encoding: string;
	media: string;
	isHearingImpaired: boolean;
	source: string;
	language: string;
	display: string;
	flagUrl: string;
}
