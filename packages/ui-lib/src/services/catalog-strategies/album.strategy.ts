import { fetchJson } from 'ui-lib/transport/fetch-helpers';
import { releaseGroupsToDisplay } from 'addons/musicbrainz/transform';
import type { MusicBrainzReleaseGroup } from 'addons/musicbrainz/types';
import type { CatalogItem, CatalogFilterOption } from 'ui-lib/types/catalog.type';
import type { CatalogKindStrategy } from 'ui-lib/services/catalog.service';

const GENRES: CatalogFilterOption[] = [
	'rock', 'pop', 'electronic', 'hip hop', 'jazz', 'classical', 'r&b', 'metal',
	'folk', 'soul', 'punk', 'blues', 'country', 'ambient', 'indie', 'alternative'
].map((g) => ({ id: g, label: g }));

function toAlbumCatalogItems(
	albums: ReturnType<typeof releaseGroupsToDisplay>
): CatalogItem[] {
	return albums.map((a) => ({
		id: a.id,
		kind: 'album' as const,
		title: a.title,
		sortTitle: a.title.toLowerCase(),
		year: a.firstReleaseYear || null,
		overview: null,
		posterUrl: a.coverArtUrl,
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
			primaryType: a.primaryType,
			secondaryTypes: a.secondaryTypes,
			artistCredits: a.artistCredits,
			firstReleaseYear: a.firstReleaseYear,
			coverArtUrl: a.coverArtUrl,
			releases: []
		}
	}));
}

export const albumStrategy: CatalogKindStrategy = {
	kind: 'album',
	tabs: [{ id: 'popular', label: 'Popular' }],
	filterDefinitions: {
		genre: { label: 'Genre', loadOptions: async () => GENRES }
	},

	async search(query, _page, _filters) {
		const data = await fetchJson<{ 'release-groups': MusicBrainzReleaseGroup[] }>(
			`/api/musicbrainz/search/release-groups?q=${encodeURIComponent(query)}`
		);
		return {
			items: toAlbumCatalogItems(releaseGroupsToDisplay(data?.['release-groups'] ?? [])),
			totalPages: 1
		};
	},

	async loadTab(_tabId, _page, filters) {
		const genre = filters.genre || 'rock';
		const data = await fetchJson<{ 'release-groups': MusicBrainzReleaseGroup[] }>(
			`/api/musicbrainz/popular?genre=${encodeURIComponent(genre)}`
		);
		return {
			items: toAlbumCatalogItems(releaseGroupsToDisplay(data?.['release-groups'] ?? [])),
			totalPages: 1
		};
	}
};
