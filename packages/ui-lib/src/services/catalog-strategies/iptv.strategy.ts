import { fetchJson } from 'ui-lib/transport/fetch-helpers';
import type {
	IptvChannel,
	IptvSearchResult,
	IptvCategory,
	IptvCountry
} from 'ui-lib/types/iptv.type';
import type { CatalogItem, CatalogFilterOption } from 'ui-lib/types/catalog.type';
import type { CatalogKindStrategy } from 'ui-lib/services/catalog.service';

function toIptvCatalogItems(channels: IptvChannel[]): CatalogItem[] {
	return channels.map((ch) => ({
		id: ch.id,
		kind: 'iptv_channel' as const,
		title: ch.name,
		sortTitle: ch.name.toLowerCase(),
		year: null,
		overview: null,
		posterUrl: ch.logo,
		backdropUrl: null,
		voteAverage: null,
		voteCount: null,
		parentId: null,
		position: null,
		source: 'iptv' as const,
		sourceId: ch.id,
		createdAt: '',
		updatedAt: '',
		metadata: {
			channelId: ch.id,
			country: ch.country,
			categories: ch.categories,
			logo: ch.logo,
			website: ch.website,
			hasEpg: ch.hasEpg,
			isNsfw: ch.isNsfw
		}
	}));
}

export const iptvStrategy: CatalogKindStrategy = {
	kind: 'iptv_channel',
	tabs: [{ id: 'browse', label: 'Channels' }],
	filterDefinitions: {
		category: {
			label: 'Category',
			loadOptions: async () => {
				const data = await fetchJson<IptvCategory[]>('/api/iptv/categories');
				return [
					{ id: '', label: 'All' },
					...(data ?? []).map((c) => ({ id: c.id, label: c.name }))
				];
			}
		},
		country: {
			label: 'Country',
			loadOptions: async () => {
				const data = await fetchJson<IptvCountry[]>('/api/iptv/countries');
				return [
					{ id: '', label: 'All' },
					...(data ?? []).map((c) => ({ id: c.code, label: c.name }))
				];
			}
		}
	},

	async search(query, page, filters) {
		const params = new URLSearchParams({ q: query, page: String(page), limit: '20' });
		if (filters.category) params.set('category', filters.category);
		if (filters.country) params.set('country', filters.country);
		const data = await fetchJson<IptvSearchResult>(`/api/iptv/channels?${params}`);
		const total = data?.total ?? 0;
		return {
			items: toIptvCatalogItems(data?.channels ?? []),
			totalPages: Math.ceil(total / 20)
		};
	},

	async loadTab(_tabId, page, filters) {
		const params = new URLSearchParams({ page: String(page), limit: '20' });
		if (filters.category) params.set('category', filters.category);
		if (filters.country) params.set('country', filters.country);
		const data = await fetchJson<IptvSearchResult>(`/api/iptv/channels?${params}`);
		const total = data?.total ?? 0;
		return {
			items: toIptvCatalogItems(data?.channels ?? []),
			totalPages: Math.ceil(total / 20)
		};
	}
};
