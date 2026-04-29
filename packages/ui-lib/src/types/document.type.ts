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
	/** Rolling-forward nonce. Older records without this field are treated as 0. */
	version?: number;
	/** CIDs of every prior version, oldest first. `version_hashes.length === version` is the chain integrity invariant. */
	version_hashes?: string[];
}
