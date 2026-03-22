export interface ImageTag {
	tag: string;
	score: number;
	confidence?: number;
}

export interface ImageItem {
	id: string;
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
	results: Record<string, ImageTag[]>;
}

export interface TaggerStatusResponse {
	ready: boolean;
	status: string;
	overallProgress: number;
	error: string | null;
}
