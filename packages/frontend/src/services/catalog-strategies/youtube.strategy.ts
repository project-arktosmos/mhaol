import { fetchJson } from '$transport/fetch-helpers';
import type { YouTubeContent } from 'addons/youtube/types';
import type { CatalogItem } from '$types/catalog.type';
import type { CatalogKindStrategy } from '$services/catalog.service';

function toYoutubeCatalogItems(items: YouTubeContent[]): CatalogItem[] {
	return items.map((v) => ({
		id: v.youtubeId,
		kind: 'youtube_video' as const,
		title: v.title,
		sortTitle: v.title.toLowerCase(),
		year: null,
		overview: null,
		posterUrl: v.thumbnailUrl,
		backdropUrl: null,
		voteAverage: null,
		voteCount: null,
		parentId: null,
		position: null,
		source: 'youtube' as const,
		sourceId: v.youtubeId,
		createdAt: v.createdAt,
		updatedAt: '',
		metadata: {
			youtubeId: v.youtubeId,
			authors: v.channelId
				? [
						{
							id: v.channelId,
							name: v.channelName ?? '',
							role: 'channel' as const,
							source: 'youtube' as const,
							imageUrl: null
						}
					]
				: [],
			durationSeconds: v.durationSeconds,
			videoPath: v.hasVideo ? 'exists' : null,
			audioPath: v.hasAudio ? 'exists' : null,
			videoSize: v.videoSize,
			audioSize: v.audioSize,
			isFavorite: v.isFavorite,
			favoritedAt: v.favoritedAt
		}
	}));
}

export const youtubeStrategy: CatalogKindStrategy = {
	kind: 'youtube_video',
	pinService: 'youtube',
	tabs: [{ id: 'library', label: 'Library' }],
	filterDefinitions: {},

	async search(query, _page, _filters) {
		const data = await fetchJson<YouTubeContent[]>('/api/youtube/content');
		const q = query.toLowerCase();
		const filtered = (data ?? []).filter((v) => v.title.toLowerCase().includes(q));
		return { items: toYoutubeCatalogItems(filtered), totalPages: 1 };
	},

	async loadTab(_tabId, _page, _filters) {
		const data = await fetchJson<YouTubeContent[]>('/api/youtube/content');
		return { items: toYoutubeCatalogItems(data ?? []), totalPages: 1 };
	},

	async resolveByIds(ids) {
		const data = await fetchJson<YouTubeContent[]>('/api/youtube/content');
		const idSet = new Set(ids);
		const matched = (data ?? []).filter((v) => idSet.has(v.youtubeId));
		return toYoutubeCatalogItems(matched);
	}
};
