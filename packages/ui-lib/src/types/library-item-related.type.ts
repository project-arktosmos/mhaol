export interface RelatedLibrary {
	id: string;
	name: string;
	path: string;
	mediaTypes: string;
	createdAt: string;
}

export interface RelatedLink {
	id: string;
	service: string;
	serviceId: string;
	seasonNumber: number | null;
	episodeNumber: number | null;
	createdAt: string;
}

export interface RelatedTorrentDownload {
	infoHash: string;
	name: string;
	size: number;
	progress: number;
	state: string;
	downloadSpeed: number;
	uploadSpeed: number;
	peers: number;
	seeds: number;
	addedAt: number;
	eta: number | null;
	outputPath: string | null;
	source: string;
	createdAt: string;
	updatedAt: string;
}

export interface RelatedFetchCache {
	tmdbId: number;
	mediaType: string;
	candidate: Record<string, unknown>;
	createdAt: string;
}

export interface RelatedTmdbCache {
	tmdbId: number;
	data: Record<string, unknown>;
	fetchedAt: string;
}

export interface LibraryItemRelated {
	library: RelatedLibrary | null;
	links: RelatedLink[];
	fetchCache: RelatedFetchCache | null;
	torrentDownload: RelatedTorrentDownload | null;
	tmdbCache: RelatedTmdbCache | null;
}
