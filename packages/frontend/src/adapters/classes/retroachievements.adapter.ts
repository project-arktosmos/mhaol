import { AdapterClass } from 'frontend/adapters/classes/adapter.class';
import type { RaGameMetadata, RaAchievement } from 'frontend/types/retroachievements.type';

const RA_MEDIA_URL = 'https://media.retroachievements.org';

function raImageUrl(path: string | null | undefined): string {
	if (!path) return '';
	return `${RA_MEDIA_URL}${path}`;
}

interface RaGameListItem {
	ID: number;
	Title: string;
	ConsoleID: number;
	ConsoleName: string;
	ImageIcon: string;
	ImageTitle: string | null;
	ImageIngame: string | null;
	ImageBoxArt: string | null;
	NumAchievements: number;
	Points: number;
	DateModified: string;
}

interface RaAchievementRaw {
	ID: number;
	Title: string;
	Description: string;
	Points: number;
	TrueRatio: number;
	BadgeName: string;
	DisplayOrder: number;
	NumAwarded: number;
	NumAwardedHardcore: number;
	Author: string;
	DateCreated: string;
	DateModified: string;
	type: string;
}

interface RaGameExtended {
	ID: number;
	Title: string;
	ConsoleID: number;
	ConsoleName: string;
	ImageIcon: string;
	ImageTitle: string | null;
	ImageIngame: string | null;
	ImageBoxArt: string | null;
	NumAchievements: number;
	Points: number;
	DateModified: string;
	Developer: string | null;
	Publisher: string | null;
	Genre: string | null;
	Released: string | null;
	NumDistinctPlayers: number;
	Achievements: Record<string, RaAchievementRaw> | null;
}

export class RetroAchievementsAdapter extends AdapterClass {
	constructor() {
		super('retroachievements');
	}

	fromGameListItem(item: RaGameListItem): RaGameMetadata {
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

	fromGameExtended(detail: RaGameExtended): RaGameMetadata {
		const achievements: RaAchievement[] = [];
		if (detail.Achievements) {
			for (const ach of Object.values(detail.Achievements)) {
				achievements.push({
					id: ach.ID,
					title: ach.Title ?? '',
					description: ach.Description ?? '',
					points: ach.Points ?? 0,
					trueRatio: ach.TrueRatio ?? 0,
					badgeUrl: ach.BadgeName ? `${RA_MEDIA_URL}/Badge/${ach.BadgeName}.png` : '',
					badgeLockedUrl: ach.BadgeName
						? `${RA_MEDIA_URL}/Badge/${ach.BadgeName}_lock.png`
						: '',
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
}

export const retroAchievementsAdapter = new RetroAchievementsAdapter();
