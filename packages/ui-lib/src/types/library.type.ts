import type { ID } from 'ui-lib/types/core.type';

export enum MediaType {
	Video = 'video',
	Image = 'image',
	Audio = 'audio',
	Document = 'document'
}

export enum LibraryType {
	Movies = 'movies',
	TV = 'tv',
	Music = 'music',
	Games = 'games',
	YouTube = 'youtube',
	Photos = 'photos',
	Books = 'books'
}

export const MEDIA_TYPE_OPTIONS: { value: MediaType; label: string }[] = [
	{ value: MediaType.Video, label: 'Video' },
	{ value: MediaType.Image, label: 'Image' },
	{ value: MediaType.Audio, label: 'Audio' },
	{ value: MediaType.Document, label: 'Document' }
];

export const LIBRARY_TYPE_OPTIONS: { value: LibraryType; label: string }[] = [
	{ value: LibraryType.Movies, label: 'Movies' },
	{ value: LibraryType.TV, label: 'TV' },
	{ value: LibraryType.Music, label: 'Music' },
	{ value: LibraryType.Games, label: 'Games' },
	{ value: LibraryType.YouTube, label: 'YouTube' },
	{ value: LibraryType.Photos, label: 'Photos' },
	{ value: LibraryType.Books, label: 'Books' }
];

export interface MediaTypeOption {
	id: string;
	label: string;
}

export interface CategoryOption {
	id: string;
	mediaTypeId: string;
	label: string;
}

export interface Library {
	id: ID;
	name: string;
	path: string;
	mediaTypes: MediaType[];
	libraryType: LibraryType;
	dateAdded: number;
}

export interface DirectoryEntry {
	name: string;
	path: string;
}

export interface BrowseDirectoryResponse {
	path: string;
	parent: string | null;
	directories: DirectoryEntry[];
}

export interface LibraryFileLink {
	serviceId: string;
	seasonNumber: number | null;
	episodeNumber: number | null;
}

export interface LibraryFile {
	id: string;
	name: string;
	path: string;
	extension: string;
	mediaType: MediaType;
	categoryId: string | null;
	links: Record<string, LibraryFileLink>;
}

export interface LibraryFilesResponse {
	libraryId: string;
	libraryPath: string;
	files: LibraryFile[];
}

export interface LibraryCardItem {
	videoId: string;
	title: string;
	thumbnailUrl: string | null;
	durationSeconds: number | null;
	channelName: string | null;
	hasVideo: boolean;
	hasAudio: boolean;
}

export interface LibraryFsEntry {
	name: string;
	size: number;
}

export interface LibraryFs {
	path: string;
	audio: LibraryFsEntry[];
	video: LibraryFsEntry[];
}
