import type { ID } from '$types/core.type';

export enum LibraryType {
	Movies = 'movies',
	TV = 'tv'
}

export const LIBRARY_TYPE_OPTIONS: { value: LibraryType; label: string }[] = [
	{ value: LibraryType.Movies, label: 'Movies' },
	{ value: LibraryType.TV, label: 'TV Shows' }
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
	mediaType: string;
	categoryId: string | null;
	links: Record<string, LibraryFileLink>;
}

export interface LibraryFilesResponse {
	libraryId: string;
	libraryPath: string;
	files: LibraryFile[];
}
