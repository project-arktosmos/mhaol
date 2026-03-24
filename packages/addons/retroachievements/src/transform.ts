import type { RaGameMetadata, RaAchievement, RaGameListItem, RaGameExtended } from './types.js';

let raMediaBaseUrl = 'https://media.retroachievements.org';

/** Set the base URL for RetroAchievements images (e.g. to route through local backend cache). */
export function setRaImageBaseUrl(url: string) {
	raMediaBaseUrl = url;
}

export function raImageUrl(path: string | null | undefined): string {
	if (!path) return '';
	return `${raMediaBaseUrl}${path}`;
}

export function gameListItemToDisplay(item: RaGameListItem): RaGameMetadata {
	return {
		id: item.ID,
		title: item.Title,
		consoleId: item.ConsoleID,
		consoleName: item.ConsoleName ?? '',
		imageIconUrl: raImageUrl(item.ImageIcon),
		imageTitleUrl: raImageUrl(item.ImageTitle),
		imageIngameUrl: raImageUrl(item.ImageIngame),
		imageBoxArtUrl: raImageUrl(item.ImageBoxArt),
		numAchievements: item.NumAchievements ?? 0,
		points: item.Points ?? 0,
		dateModified: item.DateModified ?? ''
	};
}

export function gameExtendedToDisplay(detail: RaGameExtended): RaGameMetadata {
	const achievements: RaAchievement[] = [];
	if (detail.Achievements) {
		for (const ach of Object.values(detail.Achievements)) {
			achievements.push({
				id: ach.ID,
				title: ach.Title ?? '',
				description: ach.Description ?? '',
				points: ach.Points ?? 0,
				trueRatio: ach.TrueRatio ?? 0,
				badgeUrl: ach.BadgeName ? `${raMediaBaseUrl}/Badge/${ach.BadgeName}.png` : '',
				badgeLockedUrl: ach.BadgeName ? `${raMediaBaseUrl}/Badge/${ach.BadgeName}_lock.png` : '',
				displayOrder: ach.DisplayOrder ?? 0,
				numAwarded: ach.NumAwarded ?? 0,
				numAwardedHardcore: ach.NumAwardedHardcore ?? 0,
				author: ach.Author ?? '',
				dateCreated: ach.DateCreated ?? '',
				dateModified: ach.DateModified ?? '',
				type: ach.type ?? ''
			});
		}
		achievements.sort((a, b) => a.displayOrder - b.displayOrder);
	}

	return {
		id: detail.ID,
		title: detail.Title,
		consoleId: detail.ConsoleID,
		consoleName: detail.ConsoleName ?? '',
		imageIconUrl: raImageUrl(detail.ImageIcon),
		imageTitleUrl: raImageUrl(detail.ImageTitle),
		imageIngameUrl: raImageUrl(detail.ImageIngame),
		imageBoxArtUrl: raImageUrl(detail.ImageBoxArt),
		numAchievements: detail.NumAchievements ?? 0,
		points: detail.Points ?? 0,
		dateModified: detail.DateModified ?? '',
		developer: detail.Developer ?? undefined,
		publisher: detail.Publisher ?? undefined,
		genre: detail.Genre ?? undefined,
		released: detail.Released ?? undefined,
		numDistinctPlayers: detail.NumDistinctPlayers ?? 0,
		achievements
	};
}
