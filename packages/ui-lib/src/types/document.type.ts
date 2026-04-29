export interface DocumentArtist {
	name: string;
	url?: string;
	imageUrl?: string;
}

export interface DocumentImage {
	url: string;
	mimeType: string;
	fileSize: number;
	width: number;
	height: number;
}

export type DocumentFileType = 'ipfs' | 'torrent magnet' | 'url';

export interface DocumentFile {
	type: DocumentFileType;
	value: string;
	title?: string;
}

export interface CloudDocument {
	id: string;
	title: string;
	artists: DocumentArtist[];
	description: string;
	images: DocumentImage[];
	files: DocumentFile[];
	year: number | null;
	type: string;
	source: string;
	created_at: string;
	updated_at: string;
}
