export interface MediaItemLink {
	serviceId: string;
	seasonNumber: number | null;
	episodeNumber: number | null;
}

export interface MediaItem {
	id: string;
	libraryId: string;
	name: string;
	extension: string;
	path: string;
	categoryId: string | null;
	mediaTypeId: string;
	createdAt: string;
	links: Record<string, MediaItemLink>;
}

export interface MediaLinkSource {
	id: string;
	service: string;
	label: string;
	mediaTypeId: string;
	categoryId: string | null;
}

export interface MediaCategory {
	id: string;
	mediaTypeId: string;
	label: string;
}
