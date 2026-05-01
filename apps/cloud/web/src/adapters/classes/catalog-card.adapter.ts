import type { CatalogItem, CatalogCardData, CatalogBadge } from '$types/catalog.type';
import { formatAuthors } from '$types/catalog.type';

function getSubtitle(item: CatalogItem): string | null {
	switch (item.kind) {
		case 'movie':
			return formatAuthors(item.metadata.authors, 'director') || null;
		case 'tv_show':
			return item.metadata.status;
		case 'album':
			return formatAuthors(item.metadata.authors, 'artist') || null;
		case 'youtube_video':
			return item.metadata.authors.find((a) => a.role === 'channel')?.name ?? null;
		case 'youtube_channel':
			return item.metadata.subscriberText;
		case 'photo':
			return item.metadata.tags.map((t) => t.tag).join(', ') || null;
		default:
			return null;
	}
}

function getAspectRatio(item: CatalogItem): 'poster' | 'square' | 'landscape' {
	switch (item.kind) {
		case 'movie':
		case 'tv_show':
			return 'poster';
		case 'album':
		case 'photo':
			return 'square';
		case 'youtube_video':
		case 'youtube_channel':
			return 'landscape';
		default:
			return 'poster';
	}
}

function getBadges(item: CatalogItem): CatalogBadge[] {
	const badges: CatalogBadge[] = [];
	switch (item.kind) {
		case 'movie':
		case 'tv_show':
			for (const g of (item.metadata.genres ?? []).slice(0, 2)) {
				badges.push({ label: g, variant: 'ghost' });
			}
			break;
		case 'album':
			if (item.metadata.primaryType) {
				badges.push({ label: item.metadata.primaryType, variant: 'ghost' });
			}
			break;
	}
	return badges;
}

export function catalogItemToCardData(item: CatalogItem): CatalogCardData {
	return {
		kind: item.kind,
		id: item.id,
		title: item.title,
		subtitle: getSubtitle(item),
		imageUrl: item.posterUrl,
		aspectRatio: getAspectRatio(item),
		badges: getBadges(item),
		rating: item.voteAverage,
		year: item.year
	};
}
