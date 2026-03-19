import type { MediaItem } from '$types/media-card.type';

export interface MediaListLink {
	serviceId: string;
	seasonNumber: number | null;
}

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
	links: Record<string, MediaListLink>;
	items: MediaItem[];
}
