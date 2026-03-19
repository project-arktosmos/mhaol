// MusicBrainz API response types

export interface MusicBrainzArtistCredit {
	name: string;
	joinphrase: string;
	artist: {
		id: string;
		name: string;
		'sort-name': string;
		disambiguation?: string;
	};
}

export interface MusicBrainzTag {
	count: number;
	name: string;
}

export interface MusicBrainzArtist {
	id: string;
	name: string;
	'sort-name': string;
	type?: string;
	country?: string;
	disambiguation?: string;
	'life-span'?: {
		begin?: string;
		end?: string;
		ended?: boolean;
	};
	tags?: MusicBrainzTag[];
	'release-groups'?: MusicBrainzReleaseGroup[];
	score?: number;
}

export interface MusicBrainzReleaseGroup {
	id: string;
	title: string;
	'primary-type'?: string;
	'secondary-types'?: string[];
	'first-release-date'?: string;
	'artist-credit'?: MusicBrainzArtistCredit[];
	score?: number;
}

export interface MusicBrainzRelease {
	id: string;
	title: string;
	date?: string;
	status?: string;
	country?: string;
	barcode?: string;
	'track-count'?: number;
	'label-info'?: Array<{
		'catalog-number'?: string;
		label?: { id: string; name: string };
	}>;
	'release-group'?: MusicBrainzReleaseGroup;
	'artist-credit'?: MusicBrainzArtistCredit[];
	media?: MusicBrainzMedia[];
}

export interface MusicBrainzMedia {
	position: number;
	format?: string;
	'track-count': number;
	tracks?: MusicBrainzTrack[];
}

export interface MusicBrainzTrack {
	id: string;
	number: string;
	title: string;
	length?: number;
	position: number;
	'artist-credit'?: MusicBrainzArtistCredit[];
	recording: {
		id: string;
		title: string;
		length?: number;
		disambiguation?: string;
	};
}

export interface MusicBrainzRecording {
	id: string;
	title: string;
	length?: number;
	disambiguation?: string;
	'artist-credit'?: MusicBrainzArtistCredit[];
	releases?: MusicBrainzRelease[];
	score?: number;
}

export interface MusicBrainzSearchResponse<T> {
	created: string;
	count: number;
	offset: number;
	artists?: T[];
	'release-groups'?: T[];
	releases?: T[];
	recordings?: T[];
}

export interface DisplayMusicBrainzArtist {
	id: string;
	name: string;
	sortName: string;
	type: string | null;
	country: string | null;
	disambiguation: string | null;
	beginYear: string | null;
	endYear: string | null;
	ended: boolean;
	tags: string[];
	score: number;
}

export interface DisplayMusicBrainzArtistDetails extends DisplayMusicBrainzArtist {
	releaseGroups: DisplayMusicBrainzReleaseGroup[];
	imageUrl: string | null;
}

export interface DisplayMusicBrainzReleaseGroup {
	id: string;
	title: string;
	primaryType: string | null;
	secondaryTypes: string[];
	firstReleaseYear: string;
	artistCredits: string;
	coverArtUrl: string | null;
	score: number;
}

export interface DisplayMusicBrainzRelease {
	id: string;
	title: string;
	date: string | null;
	status: string | null;
	country: string | null;
	artistCredits: string;
	trackCount: number;
	label: string | null;
	tracks: DisplayMusicBrainzTrack[];
}

export interface DisplayMusicBrainzTrack {
	id: string;
	number: string;
	title: string;
	duration: string | null;
	durationMs: number | null;
	artistCredits: string;
}

export interface DisplayMusicBrainzRecording {
	id: string;
	title: string;
	duration: string | null;
	durationMs: number | null;
	artistCredits: string;
	disambiguation: string | null;
	coverArtUrl: string | null;
	firstReleaseTitle: string | null;
	score: number;
}
