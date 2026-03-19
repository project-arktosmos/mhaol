export interface CloudLibrary {
	id: string;
	name: string;
	path: string;
	kind: string;
	scanStatus: 'idle' | 'scanning' | 'error';
	scanError: string | null;
	itemCount: number;
}

export interface CloudItemAttribute {
	key: string;
	value: string;
	typeId: string;
	source: string;
	confidence: number | null;
}

export interface CloudItemLink {
	service: string;
	serviceId: string;
	extra: string | null;
}

export interface CloudItem {
	id: string;
	libraryId: string;
	path: string;
	filename: string;
	extension: string;
	sizeBytes: number | null;
	mimeType: string | null;
	attributes: CloudItemAttribute[];
	links: CloudItemLink[];
}

export interface CloudCollection {
	id: string;
	libraryId: string;
	name: string;
	description: string | null;
	coverPath: string | null;
	kind: 'manual' | 'auto' | 'smart';
	itemCount: number;
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

export interface CloudScanResponse {
	libraryId: string;
	libraryPath: string;
	itemCount: number;
	items: CloudItem[];
}
