import { fetchJson } from 'ui-lib/transport/fetch-helpers';
import { tvShowsToDisplay } from 'addons/tmdb/transform';
import type { TMDBTvShow } from 'addons/tmdb/types';
import type { CatalogItem, CatalogFilterOption } from 'ui-lib/types/catalog.type';
import type { CatalogKindStrategy } from 'ui-lib/services/catalog.service';

interface TmdbPagedResponse {
	results: TMDBTvShow[];
	total_pages: number;
	page: number;
}

function toTvCatalogItems(shows: TMDBTvShow[]): CatalogItem[] {
	return tvShowsToDisplay(shows).map((s) => ({
		id: String(s.id),
		kind: 'tv_show' as const,
		title: s.name,
		sortTitle: s.name.toLowerCase(),
		year: s.firstAirYear || null,
		overview: s.overview || null,
		posterUrl: s.posterUrl,
		backdropUrl: s.backdropUrl,
		voteAverage: s.voteAverage,
		voteCount: s.voteCount,
		parentId: null,
		position: null,
		source: 'tmdb' as const,
		sourceId: String(s.id),
		createdAt: '',
		updatedAt: '',
		metadata: {
			tmdbId: s.id,
			originalName: s.originalName,
			lastAirYear: s.lastAirYear,
			status: null,
			networks: [],
			createdBy: [],
			cast: [],
			genres: s.genres,
			tagline: null,
			numberOfSeasons: s.numberOfSeasons,
			numberOfEpisodes: s.numberOfEpisodes,
			seasons: [],
			images: [],
			imageOverrides: {}
		}
	}));
}

async function loadGenres(): Promise<CatalogFilterOption[]> {
	const data = await fetchJson<{ genres: { id: number; name: string }[] }>('/api/tmdb/genres/tv');
	return (data?.genres ?? []).map((g) => ({ id: String(g.id), label: g.name }));
}

export const tvStrategy: CatalogKindStrategy = {
	kind: 'tv_show',
	pinService: 'tmdb-tv',
	tabs: [
		{ id: 'popular', label: 'Popular' },
		{ id: 'discover', label: 'Discover' }
	],
	filterDefinitions: {
		genre: { label: 'Genre', loadOptions: loadGenres }
	},

	async search(query, page, _filters) {
		const data = await fetchJson<TmdbPagedResponse>(
			`/api/tmdb/search/tv?query=${encodeURIComponent(query)}&page=${page}`
		);
		return {
			items: toTvCatalogItems(data?.results ?? []),
			totalPages: data?.total_pages ?? 1
		};
	},

	async loadTab(tabId, page, filters) {
		let url: string;
		if (tabId === 'discover' && filters.genre) {
			url = `/api/tmdb/discover/tv?page=${page}&with_genres=${filters.genre}`;
		} else if (tabId === 'discover') {
			url = `/api/tmdb/discover/tv?page=${page}`;
		} else {
			url = `/api/tmdb/tv/popular?page=${page}`;
		}
		const data = await fetchJson<TmdbPagedResponse>(url);
		return {
			items: toTvCatalogItems(data?.results ?? []),
			totalPages: data?.total_pages ?? 1
		};
	},

	async resolveByIds(ids) {
		const results = await Promise.allSettled(
			ids.map((id) => fetchJson<TMDBTvShow>(`/api/tmdb/tv/${id}`))
		);
		return results
			.filter(
				(r): r is PromiseFulfilledResult<TMDBTvShow> =>
					r.status === 'fulfilled' && r.value != null
			)
			.flatMap((r) => toTvCatalogItems([r.value]));
	}
};
