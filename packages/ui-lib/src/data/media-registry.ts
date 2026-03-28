import type { CatalogKind, CatalogItem } from 'ui-lib/types/catalog.type';
import { isBook } from 'ui-lib/types/catalog.type';

export type FilterKind =
	| 'genre-buttons'
	| 'subject-buttons'
	| 'console-selector'
	| 'category-country'
	| 'none';

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
	books: {
		slug: 'books',
		kind: 'book',
		label: 'Books',
		pinService: 'openlibrary',
		favService: 'openlibrary',
		filterKind: 'subject-buttons',
		hasRecs: true,
		recsMediaType: 'book',
		features: {},
		selectItemId: (item) => (isBook(item) ? item.metadata.openlibraryKey : item.sourceId)
	},
	videogames: {
		slug: 'videogames',
		kind: 'game',
		label: 'Videogames',
		pinService: 'retroachievements',
		favService: 'retroachievements',
		filterKind: 'console-selector',
		hasRecs: true,
		recsMediaType: 'game',
		features: {},
		selectItemId: (item) => item.sourceId
	},
	iptv: {
		slug: 'iptv',
		kind: 'iptv_channel',
		label: 'IPTV',
		pinService: 'iptv',
		favService: 'iptv',
		filterKind: 'category-country',
		hasRecs: false,
		features: {},
		encodeId: true,
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
