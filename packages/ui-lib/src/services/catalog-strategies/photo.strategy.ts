import { fetchJson } from 'ui-lib/transport/fetch-helpers';
import type { CatalogItem } from 'ui-lib/types/catalog.type';
import type { CatalogKindStrategy } from 'ui-lib/services/catalog.service';

interface PhotoItem {
	id: string;
	libraryId: string;
	name: string;
	path: string;
	extension: string;
	tags: { tag: string; score: number }[];
}

function toPhotoCatalogItems(photos: PhotoItem[]): CatalogItem[] {
	return photos.map((p) => ({
		id: p.id,
		kind: 'photo' as const,
		title: p.name,
		sortTitle: p.name.toLowerCase(),
		year: null,
		overview: null,
		posterUrl: `/api/images/thumbnail/${p.id}`,
		backdropUrl: null,
		voteAverage: null,
		voteCount: null,
		parentId: null,
		position: null,
		source: 'local' as const,
		sourceId: p.id,
		createdAt: '',
		updatedAt: '',
		metadata: {
			libraryItemId: p.id,
			libraryId: p.libraryId,
			path: p.path,
			extension: p.extension,
			tags: p.tags
		}
	}));
}

export const photoStrategy: CatalogKindStrategy = {
	kind: 'photo',
	tabs: [{ id: 'all', label: 'All' }],
	filterDefinitions: {},

	async search(query, _page, _filters) {
		const data = await fetchJson<PhotoItem[]>('/api/media?type=image');
		const q = query.toLowerCase();
		const filtered = (data ?? []).filter(
			(p) => p.name.toLowerCase().includes(q) || p.tags.some((t) => t.tag.toLowerCase().includes(q))
		);
		return { items: toPhotoCatalogItems(filtered), totalPages: 1 };
	},

	async loadTab(_tabId, _page, _filters) {
		const data = await fetchJson<PhotoItem[]>('/api/media?type=image');
		return { items: toPhotoCatalogItems(data ?? []), totalPages: 1 };
	}
};
