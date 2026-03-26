import { fetchJson } from 'ui-lib/transport/fetch-helpers';
import { artistsToDisplay } from 'addons/musicbrainz/transform';
import type { MusicBrainzArtist } from 'addons/musicbrainz/types';
import type { CatalogItem } from 'ui-lib/types/catalog.type';
import type { CatalogKindStrategy } from 'ui-lib/services/catalog.service';

function toArtistCatalogItems(
	artists: ReturnType<typeof artistsToDisplay>
): CatalogItem[] {
	return artists.map((a) => ({
		id: a.id,
		kind: 'artist' as const,
		title: a.name,
		sortTitle: a.sortName.toLowerCase(),
		year: a.beginYear || null,
		overview: null,
		posterUrl: a.imageUrl,
		backdropUrl: null,
		voteAverage: null,
		voteCount: null,
		parentId: null,
		position: null,
		source: 'musicbrainz' as const,
		sourceId: a.id,
		createdAt: '',
		updatedAt: '',
		metadata: {
			musicbrainzId: a.id,
			sortName: a.sortName,
			type: a.type,
			country: a.country,
			disambiguation: a.disambiguation,
			beginYear: a.beginYear,
			endYear: a.endYear,
			ended: a.ended,
			tags: a.tags,
			imageUrl: a.imageUrl
		}
	}));
}

export const artistStrategy: CatalogKindStrategy = {
	kind: 'artist',
	tabs: [{ id: 'popular', label: 'Popular' }],
	filterDefinitions: {},

	async search(query, _page, _filters) {
		const data = await fetchJson<{ artists: MusicBrainzArtist[] }>(
			`/api/musicbrainz/search/artists?q=${encodeURIComponent(query)}`
		);
		return {
			items: toArtistCatalogItems(artistsToDisplay(data?.artists ?? [])),
			totalPages: 1
		};
	},

	async loadTab(_tabId, _page, _filters) {
		const data = await fetchJson<{ artists: MusicBrainzArtist[] }>(
			'/api/musicbrainz/popular/artists'
		);
		return {
			items: toArtistCatalogItems(artistsToDisplay(data?.artists ?? [])),
			totalPages: 1
		};
	}
};
