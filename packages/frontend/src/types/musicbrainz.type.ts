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
	'release-group'?: MusicBrainzReleaseGroup;
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

export interface DisplayMusicBrainzRecording {
	id: string;
	title: string;
	duration: string | null;
	artistCredits: string;
	disambiguation: string | null;
	coverArtUrl: string | null;
	firstReleaseTitle: string | null;
}

export interface DisplayMusicBrainzArtist {
	id: string;
	name: string;
	type: string | null;
	country: string | null;
	disambiguation: string | null;
	beginYear: string | null;
	endYear: string | null;
	tags: string[];
}

export interface DisplayMusicBrainzReleaseGroup {
	id: string;
	title: string;
	primaryType: string | null;
	firstReleaseYear: string;
	artistCredits: string;
	coverArtUrl: string | null;
}
