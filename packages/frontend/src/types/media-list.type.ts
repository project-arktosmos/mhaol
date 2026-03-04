import type { MediaItem } from '$types/media-card.type';

export interface MediaList {
	id: string;
	libraryId: string;
	title: string;
	description: string | null;
	coverImage: string | null;
	mediaType: string;
	source: 'auto' | 'user';
	itemCount: number;
	createdAt: string;
	items: MediaItem[];
}
