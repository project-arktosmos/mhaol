import type { CatalogItem, CatalogCardData, CatalogBadge } from 'ui-lib/types/catalog.type';
import { formatAuthors } from 'ui-lib/types/catalog.type';
import { CONSOLE_WASM_STATUS } from 'addons/retroachievements/types';

function getSubtitle(item: CatalogItem): string | null {
	switch (item.kind) {
		case 'movie':
			return formatAuthors(item.metadata.authors, 'director') || null;
		case 'tv_show':
			return item.metadata.status;
		case 'album':
			return formatAuthors(item.metadata.authors, 'artist') || null;
		case 'book':
			return formatAuthors(item.metadata.authors, 'author') || null;
		case 'game':
			return item.metadata.consoleName;
		case 'youtube_video':
			return item.metadata.authors.find((a) => a.role === 'channel')?.name ?? null;
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
		case 'book':
			return 'poster';
		case 'album':
		case 'game':
		case 'photo':
			return 'square';
		case 'youtube_video':
		case 'youtube_channel':
		case 'iptv_channel':
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
		case 'game': {
			const wasm = CONSOLE_WASM_STATUS[item.metadata.consoleId];
			if (wasm === 'yes') {
				badges.push({ label: 'Play', variant: 'success' });
			} else if (wasm === 'experimental') {
				badges.push({ label: 'Play (beta)', variant: 'warning' });
			}
			if (item.metadata.numAchievements > 0) {
				badges.push({
					label: `${item.metadata.numAchievements} achievements`,
					variant: 'accent'
				});
			}
			break;
		}
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
