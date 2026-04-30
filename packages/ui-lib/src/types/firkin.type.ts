export interface FirkinArtist {
	name: string;
	url?: string;
	imageUrl?: string;
}

export interface FirkinImage {
	url: string;
	mimeType: string;
	fileSize: number;
	width: number;
	height: number;
}

export type FirkinFileType = 'ipfs' | 'torrent magnet' | 'url';

export interface FirkinFile {
	type: FirkinFileType;
	value: string;
	title?: string;
}

export interface CloudFirkin {
	id: string;
	title: string;
	artists: FirkinArtist[];
	description: string;
	images: FirkinImage[];
	files: FirkinFile[];
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
