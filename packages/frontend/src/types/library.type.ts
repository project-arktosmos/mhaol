import type { ID } from '$types/core.type';

export enum MediaType {
	Video = 'video',
	Images = 'images',
	Music = 'music'
}

export const MEDIA_TYPE_OPTIONS: { value: MediaType; label: string }[] = [
	{ value: MediaType.Video, label: 'Video' },
	{ value: MediaType.Images, label: 'Images' },
	{ value: MediaType.Music, label: 'Music' }
];

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

export interface LibraryFile {
	name: string;
	path: string;
	size: number;
	extension: string;
	mediaType: MediaType;
}

export interface LibraryFilesResponse {
	libraryId: string;
	libraryPath: string;
	files: LibraryFile[];
}
