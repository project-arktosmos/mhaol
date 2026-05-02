import { fetchJson } from '$transport/fetch-helpers';
import { moviesToDisplay } from 'addons/tmdb/transform';
import type { TMDBMovie } from 'addons/tmdb/types';
import type { CatalogItem, CatalogFilterOption } from '$types/catalog.type';
import type { CatalogKindStrategy } from '$services/catalog.service';

interface TmdbPagedResponse {
	results: TMDBMovie[];
	total_pages: number;
	page: number;
}

function toMovieCatalogItems(movies: TMDBMovie[]): CatalogItem[] {
	return moviesToDisplay(movies).map((m) => ({
		id: String(m.id),
		kind: 'movie' as const,
		title: m.title,
		sortTitle: m.title.toLowerCase(),
		year: m.releaseYear || null,
		overview: m.overview || null,
		posterUrl: m.posterUrl,
		backdropUrl: m.backdropUrl,
		voteAverage: m.voteAverage,
		voteCount: m.voteCount,
		parentId: null,
		position: null,
		source: 'tmdb' as const,
		sourceId: String(m.id),
		createdAt: '',
		updatedAt: '',
		metadata: {
			tmdbId: m.id,
			originalTitle: m.originalTitle,
			runtime: null,
			authors: [],
			genres: m.genres,
			tagline: null,
			budget: null,
			revenue: null,
			imdbId: null,
			images: [],
			imageOverrides: {}
		}
	}));
}

async function loadGenres(): Promise<CatalogFilterOption[]> {
	const data = await fetchJson<{ genres: { id: number; name: string }[] }>(
		'/api/tmdb/genres/movie'
	);
	return (data?.genres ?? []).map((g) => ({ id: String(g.id), label: g.name }));
}

export const movieStrategy: CatalogKindStrategy = {
	kind: 'movie',
	pinService: 'tmdb',
	tabs: [
		{ id: 'popular', label: 'Popular' },
		{ id: 'discover', label: 'Discover' }
	],
	filterDefinitions: {
		genre: { label: 'Genre', loadOptions: loadGenres }
	},

	async search(query, page, _filters) {
		const data = await fetchJson<TmdbPagedResponse>(
			`/api/tmdb/search/movies?q=${encodeURIComponent(query)}&page=${page}`
		);
		return {
			items: toMovieCatalogItems(data?.results ?? []),
			totalPages: data?.total_pages ?? 1
		};
	},

	async loadTab(tabId, page, filters) {
		let url: string;
		if (tabId === 'discover' && filters.genre) {
			url = `/api/tmdb/discover/movies?page=${page}&with_genres=${filters.genre}`;
		} else if (tabId === 'discover') {
			url = `/api/tmdb/discover/movies?page=${page}`;
		} else {
			url = `/api/tmdb/popular/movies?page=${page}`;
		}
		const data = await fetchJson<TmdbPagedResponse>(url);
		return {
			items: toMovieCatalogItems(data?.results ?? []),
			totalPages: data?.total_pages ?? 1
		};
	},

	async resolveByIds(ids) {
		const results = await Promise.allSettled(
			ids.map((id) => fetchJson<TMDBMovie>(`/api/tmdb/movies/${id}`))
		);
		return results
			.filter(
				(r): r is PromiseFulfilledResult<TMDBMovie> => r.status === 'fulfilled' && r.value != null
			)
			.flatMap((r) => toMovieCatalogItems([r.value]));
	}
};
