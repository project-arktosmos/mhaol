export interface TagResult {
	tag: string;
	score: number;
}

export interface TaggerProgress {
	status: 'idle' | 'downloading' | 'loading' | 'ready' | 'error';
	files: Record<string, { loaded: number; total: number; progress: number }>;
	overallProgress: number;
	error: string | null;
}

export interface ImageTag {
	tag: string;
	score: number;
}

export interface ImageItem {
	id: string;
	libraryId: string;
	libraryName: string;
	name: string;
	path: string;
	extension: string;
	tags: ImageTag[];
}

export interface ImagesResponse {
	images: ImageItem[];
}

export interface TagResponse {
	libraryItemId: string;
	tags: ImageTag[];
}

export interface BatchTagResponse {
	results: Record<string, ImageTag[]>;
}

export interface TaggerStatusResponse {
	ready: boolean;
	status: 'idle' | 'downloading' | 'loading' | 'ready' | 'error';
	overallProgress: number;
	error: string | null;
}
