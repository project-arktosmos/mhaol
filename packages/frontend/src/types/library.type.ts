import type { ID } from '$types/core.type';

export enum MediaType {
	Video = 'video',
	Image = 'image',
	Audio = 'audio'
}

export const MEDIA_TYPE_OPTIONS: { value: MediaType; label: string }[] = [
	{ value: MediaType.Video, label: 'Video' },
	{ value: MediaType.Image, label: 'Image' },
	{ value: MediaType.Audio, label: 'Audio' }
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
