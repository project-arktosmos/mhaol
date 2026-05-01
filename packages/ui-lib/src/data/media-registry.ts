import type { CatalogKind, CatalogItem } from 'ui-lib/types/catalog.type';

export type FilterKind = 'genre-buttons' | 'none';

export interface MediaTypeConfig {
	slug: string;
	kind: CatalogKind;
	label: string;
	pinService: string;
	favService: string;
	filterKind: FilterKind;
	hasRecs: boolean;
	recsMediaType?: string;
	features: {
		libraryItems?: 'movie' | 'tv';
		fetchCache?: boolean;
		batchSmartSearch?: boolean;
		imageOverrides?: 'movie' | 'tv';
	};
	encodeId?: boolean;
	selectItemId: (item: CatalogItem) => string;
}

export const MEDIA_REGISTRY: Record<string, MediaTypeConfig> = {
	movies: {
		slug: 'movies',
		kind: 'movie',
		label: 'Movies',
		pinService: 'tmdb',
		favService: 'tmdb',
		filterKind: 'genre-buttons',
		hasRecs: true,
		recsMediaType: 'movie',
		features: {
			libraryItems: 'movie',
			fetchCache: true,
			batchSmartSearch: true,
			imageOverrides: 'movie'
		},
		selectItemId: (item) => item.sourceId
	},
	tv: {
		slug: 'tv',
		kind: 'tv_show',
		label: 'TV Shows',
		pinService: 'tmdb-tv',
		favService: 'tmdb-tv',
		filterKind: 'genre-buttons',
		hasRecs: true,
		recsMediaType: 'tv',
		features: {
			libraryItems: 'tv'
		},
		selectItemId: (item) => item.sourceId
	},
	music: {
		slug: 'music',
		kind: 'album',
		label: 'Music',
		pinService: 'musicbrainz-album',
		favService: 'musicbrainz-album',
		filterKind: 'genre-buttons',
		hasRecs: true,
		recsMediaType: 'music',
		features: {},
		selectItemId: (item) => item.sourceId
	}
};

export function getMediaConfig(slug: string): MediaTypeConfig | null {
	return MEDIA_REGISTRY[slug] ?? null;
}
