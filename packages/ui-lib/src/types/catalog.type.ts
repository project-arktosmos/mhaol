// === Kind & Source ===

export type CatalogKind =
	| 'movie'
	| 'tv_show'
	| 'tv_season'
	| 'tv_episode'
	| 'artist'
	| 'album'
	| 'track'
	| 'book'
	| 'game'
	| 'youtube_video'
	| 'youtube_channel'
	| 'iptv_channel'
	| 'photo';

export type CatalogSource =
	| 'tmdb'
	| 'musicbrainz'
	| 'openlibrary'
	| 'retroachievements'
	| 'youtube'
	| 'iptv'
	| 'local';

// === Authors ===

export type AuthorRole =
	| 'director'
	| 'actor'
	| 'writer'
	| 'creator'
	| 'producer'
	| 'artist'
	| 'author'
	| 'developer'
	| 'publisher'
	| 'channel';

export interface CatalogAuthor {
	id: string;
	name: string;
	role: AuthorRole;
	source: CatalogSource;
	imageUrl: string | null;
	character?: string;
	joinPhrase?: string;
	bio?: string;
	birthDate?: string;
	deathDate?: string;
}

export function formatAuthors(authors: CatalogAuthor[], role?: AuthorRole): string {
	const filtered = role ? authors.filter((a) => a.role === role) : authors;
	return filtered.map((a) => a.name + (a.joinPhrase ?? '')).join('') || '';
}

export function authorsByRole(authors: CatalogAuthor[], role: AuthorRole): CatalogAuthor[] {
	return authors.filter((a) => a.role === role);
}

export function primaryAuthor(authors: CatalogAuthor[]): CatalogAuthor | null {
	const priority: AuthorRole[] = ['director', 'creator', 'author', 'artist', 'developer', 'channel'];
	for (const role of priority) {
		const found = authors.find((a) => a.role === role);
		if (found) return found;
	}
	return authors[0] ?? null;
}

// === Base ===

export interface CatalogItemBase {
	id: string;
	kind: CatalogKind;
	title: string;
	sortTitle: string;
	year: string | null;
	overview: string | null;
	posterUrl: string | null;
	backdropUrl: string | null;
	voteAverage: number | null;
	voteCount: number | null;
	parentId: string | null;
	position: number | null;
	source: CatalogSource;
	sourceId: string;
	createdAt: string;
	updatedAt: string;
}

// === Per-kind items ===

export interface CatalogMovie extends CatalogItemBase {
	kind: 'movie';
	metadata: MovieMetadata;
}

export interface MovieMetadata {
	tmdbId: number;
	originalTitle: string;
	runtime: string | null;
	authors: CatalogAuthor[];
	genres: string[];
	tagline: string | null;
	budget: string | null;
	revenue: string | null;
	imdbId: string | null;
	images: CatalogImage[];
	imageOverrides: Record<string, string>;
}

export interface CatalogTvShow extends CatalogItemBase {
	kind: 'tv_show';
	metadata: TvShowMetadata;
}

export interface TvShowMetadata {
	tmdbId: number;
	originalName: string;
	lastAirYear: string | null;
	status: string | null;
	networks: string[];
	authors: CatalogAuthor[];
	genres: string[];
	tagline: string | null;
	numberOfSeasons: number | null;
	numberOfEpisodes: number | null;
	seasons: CatalogTvSeasonSummary[];
	images: CatalogImage[];
	imageOverrides: Record<string, string>;
}

export interface CatalogTvSeasonSummary {
	id: number;
	name: string;
	overview: string;
	airDate: string | null;
	episodeCount: number;
	posterUrl: string | null;
	seasonNumber: number;
}

export interface CatalogTvSeason extends CatalogItemBase {
	kind: 'tv_season';
	metadata: TvSeasonMetadata;
}

export interface TvSeasonMetadata {
	tmdbId: number;
	seasonNumber: number;
	episodeCount: number;
	airDate: string | null;
	episodes: CatalogTvEpisodeSummary[];
}

export interface CatalogTvEpisodeSummary {
	id: number;
	name: string;
	overview: string;
	airDate: string | null;
	episodeNumber: number;
	seasonNumber: number;
	stillUrl: string | null;
	runtime: number | null;
	voteAverage: number;
}

export interface CatalogTvEpisode extends CatalogItemBase {
	kind: 'tv_episode';
	metadata: TvEpisodeMetadata;
}

export interface TvEpisodeMetadata {
	tmdbId: number;
	seasonNumber: number;
	episodeNumber: number;
	airDate: string | null;
	runtime: number | null;
	stillUrl: string | null;
}

export interface CatalogArtist extends CatalogItemBase {
	kind: 'artist';
	metadata: ArtistMetadata;
}

export interface ArtistMetadata {
	musicbrainzId: string;
	sortName: string;
	type: string | null;
	country: string | null;
	disambiguation: string | null;
	beginYear: string | null;
	endYear: string | null;
	ended: boolean;
	tags: string[];
	imageUrl: string | null;
}

export interface CatalogAlbum extends CatalogItemBase {
	kind: 'album';
	metadata: AlbumMetadata;
}

export interface AlbumMetadata {
	musicbrainzId: string;
	primaryType: string | null;
	secondaryTypes: string[];
	authors: CatalogAuthor[];
	firstReleaseYear: string;
	coverArtUrl: string | null;
	releases: AlbumRelease[];
}

export interface AlbumRelease {
	id: string;
	title: string;
	date: string | null;
	status: string | null;
	country: string | null;
	authors: CatalogAuthor[];
	trackCount: number;
	label: string | null;
	tracks: AlbumTrack[];
}

export interface AlbumTrack {
	id: string;
	number: string;
	title: string;
	duration: string | null;
	durationMs: number | null;
	authors: CatalogAuthor[];
}

export interface CatalogTrack extends CatalogItemBase {
	kind: 'track';
	metadata: TrackMetadata;
}

export interface TrackMetadata {
	musicbrainzId: string;
	number: string;
	duration: string | null;
	durationMs: number | null;
	authors: CatalogAuthor[];
	disambiguation: string | null;
}

export interface CatalogBook extends CatalogItemBase {
	kind: 'book';
	metadata: BookMetadata;
}

export interface BookMetadata {
	openlibraryKey: string;
	authors: CatalogAuthor[];
	firstPublishYear: string;
	coverId: number | null;
	coverUrl: string | null;
	subjects: string[];
	publishers: string[];
	pageCount: number | null;
	editionCount: number;
	isbn: string | null;
	ratingsAverage: number | null;
	ratingsCount: number;
	description: string | null;
}

export interface CatalogGame extends CatalogItemBase {
	kind: 'game';
	metadata: GameMetadata;
}

export interface GameMetadata {
	retroachievementsId: number;
	consoleId: number;
	consoleName: string;
	imageIconUrl: string;
	numAchievements: number;
	points: number;
	authors: CatalogAuthor[];
	genre: string | null;
	released: string | null;
	imageTitleUrl: string | null;
	imageIngameUrl: string | null;
	imageBoxArtUrl: string | null;
	achievements: GameAchievement[];
}

export interface GameAchievement {
	id: number;
	title: string;
	description: string;
	points: number;
	trueRatio: number;
	badgeUrl: string;
	displayOrder: number;
	numAwarded: number;
	numAwardedHardcore: number;
}

export interface CatalogYoutubeVideo extends CatalogItemBase {
	kind: 'youtube_video';
	metadata: YoutubeVideoMetadata;
}

export interface YoutubeVideoMetadata {
	youtubeId: string;
	authors: CatalogAuthor[];
	durationSeconds: number | null;
	videoPath: string | null;
	audioPath: string | null;
	videoSize: number | null;
	audioSize: number | null;
	isFavorite: boolean;
	favoritedAt: string | null;
}

export interface CatalogYoutubeChannel extends CatalogItemBase {
	kind: 'youtube_channel';
	metadata: YoutubeChannelMetadata;
}

export interface YoutubeChannelMetadata {
	channelId: string;
	handle: string;
	url: string;
	subscriberText: string | null;
	imageUrl: string | null;
}

export interface CatalogIptvChannel extends CatalogItemBase {
	kind: 'iptv_channel';
	metadata: IptvChannelMetadata;
}

export interface IptvChannelMetadata {
	channelId: string;
	country: string;
	categories: string[];
	logo: string | null;
	website: string | null;
	hasEpg: boolean;
	isNsfw: boolean;
}

export interface CatalogPhoto extends CatalogItemBase {
	kind: 'photo';
	metadata: PhotoMetadata;
}

export interface PhotoMetadata {
	libraryItemId: string;
	libraryId: string;
	path: string;
	extension: string;
	tags: PhotoTag[];
}

export interface PhotoTag {
	tag: string;
	score: number;
}

// === Shared sub-types ===

export interface CatalogImage {
	thumbnailUrl: string;
	fullUrl: string;
	width: number;
	height: number;
	filePath: string;
	imageType: 'poster' | 'backdrop';
}

// === Discriminated union ===

export type CatalogItem =
	| CatalogMovie
	| CatalogTvShow
	| CatalogTvSeason
	| CatalogTvEpisode
	| CatalogArtist
	| CatalogAlbum
	| CatalogTrack
	| CatalogBook
	| CatalogGame
	| CatalogYoutubeVideo
	| CatalogYoutubeChannel
	| CatalogIptvChannel
	| CatalogPhoto;

// === Type guards ===

export function isMovie(item: CatalogItem): item is CatalogMovie {
	return item.kind === 'movie';
}
export function isTvShow(item: CatalogItem): item is CatalogTvShow {
	return item.kind === 'tv_show';
}
export function isTvSeason(item: CatalogItem): item is CatalogTvSeason {
	return item.kind === 'tv_season';
}
export function isTvEpisode(item: CatalogItem): item is CatalogTvEpisode {
	return item.kind === 'tv_episode';
}
export function isArtist(item: CatalogItem): item is CatalogArtist {
	return item.kind === 'artist';
}
export function isAlbum(item: CatalogItem): item is CatalogAlbum {
	return item.kind === 'album';
}
export function isTrack(item: CatalogItem): item is CatalogTrack {
	return item.kind === 'track';
}
export function isBook(item: CatalogItem): item is CatalogBook {
	return item.kind === 'book';
}
export function isGame(item: CatalogItem): item is CatalogGame {
	return item.kind === 'game';
}
export function isYoutubeVideo(item: CatalogItem): item is CatalogYoutubeVideo {
	return item.kind === 'youtube_video';
}
export function isYoutubeChannel(item: CatalogItem): item is CatalogYoutubeChannel {
	return item.kind === 'youtube_channel';
}
export function isIptvChannel(item: CatalogItem): item is CatalogIptvChannel {
	return item.kind === 'iptv_channel';
}
export function isPhoto(item: CatalogItem): item is CatalogPhoto {
	return item.kind === 'photo';
}

// === Card rendering ===

export interface CatalogCardData {
	kind: CatalogKind;
	id: string;
	title: string;
	subtitle: string | null;
	imageUrl: string | null;
	aspectRatio: 'poster' | 'square' | 'landscape';
	badges: CatalogBadge[];
	rating: number | null;
	year: string | null;
	favorited?: boolean;
	pinned?: boolean;
	fetched?: boolean;
	selected?: boolean;
	dimmed?: boolean;
	loading?: boolean;
	torrentProgress?: number;
	torrentState?: string;
	fetchCacheSummary?: string;
}

export interface CatalogBadge {
	label: string;
	variant: string;
}

// === Browse state ===

export interface CatalogTab {
	id: string;
	label: string;
}

export interface CatalogFilterOption {
	id: string;
	label: string;
}

export interface CatalogBrowseState {
	kind: CatalogKind;
	items: CatalogItem[];
	loading: boolean;
	error: string | null;
	searchQuery: string;
	page: number;
	totalPages: number;
	activeTab: string;
	tabs: CatalogTab[];
	filters: Record<string, string>;
	filterOptions: Record<string, CatalogFilterOption[]>;
}

// === Fetch cache ===

export interface CatalogFetchCacheEntry {
	id: string;
	catalogItemId: string;
	scope: string;
	scopeKey: string;
	candidate: Record<string, unknown>;
	createdAt: string;
}

// === Kind groupings ===

export const BROWSE_KINDS: CatalogKind[] = [
	'movie',
	'tv_show',
	'album',
	'artist',
	'book',
	'game',
	'youtube_video',
	'iptv_channel',
	'photo'
];

export const TORRENT_KINDS: CatalogKind[] = ['movie', 'tv_show', 'album', 'book', 'game'];

export const STREAMABLE_KINDS: CatalogKind[] = [
	'youtube_video',
	'iptv_channel',
	'movie',
	'tv_episode'
];
