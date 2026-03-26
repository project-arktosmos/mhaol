import type { CatalogItem, CatalogCardData, CatalogBadge } from 'ui-lib/types/catalog.type';

function getSubtitle(item: CatalogItem): string | null {
	switch (item.kind) {
		case 'movie':
			return item.metadata.director;
		case 'tv_show':
			return item.metadata.status;
		case 'album':
			return item.metadata.artistCredits;
		case 'artist':
			return item.metadata.country;
		case 'track':
			return item.metadata.artistCredits;
		case 'book':
			return item.metadata.authors.join(', ') || null;
		case 'game':
			return item.metadata.consoleName;
		case 'youtube_video':
			return item.metadata.channelName;
		case 'youtube_channel':
			return item.metadata.subscriberText;
		case 'iptv_channel':
			return item.metadata.country;
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
		case 'tv_season':
		case 'book':
			return 'poster';
		case 'album':
		case 'artist':
		case 'game':
		case 'photo':
			return 'square';
		case 'youtube_video':
		case 'youtube_channel':
		case 'iptv_channel':
		case 'tv_episode':
		case 'track':
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
		case 'artist':
			if (item.metadata.type) {
				badges.push({ label: item.metadata.type, variant: 'info' });
			}
			break;
		case 'game':
			if (item.metadata.numAchievements > 0) {
				badges.push({
					label: `${item.metadata.numAchievements} achievements`,
					variant: 'accent'
				});
			}
			break;
		case 'iptv_channel':
			for (const c of item.metadata.categories.slice(0, 1)) {
				badges.push({ label: c, variant: 'info' });
			}
			if (item.metadata.hasEpg) {
				badges.push({ label: 'EPG', variant: 'success' });
			}
			break;
		case 'book':
			for (const s of item.metadata.subjects.slice(0, 2)) {
				badges.push({ label: s, variant: 'ghost' });
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
