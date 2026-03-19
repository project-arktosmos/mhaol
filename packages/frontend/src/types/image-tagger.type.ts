export interface ImageTag {
	tag: string;
	confidence: number;
	score?: number;
}

export interface ImageItem {
	path: string;
	tags: ImageTag[];
}

export interface ImagesResponse {
	images: ImageItem[];
}

export interface TagResponse {
	tags: ImageTag[];
}

export interface BatchTagResponse {
	results: Array<{ path: string; tags: ImageTag[] }>;
}

export interface TaggerStatusResponse {
	available: boolean;
	model?: string;
}
