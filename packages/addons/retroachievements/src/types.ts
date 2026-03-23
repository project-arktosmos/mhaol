export interface RaAchievement {
	id: number;
	title: string;
	description: string;
	points: number;
	trueRatio: number;
	badgeUrl: string;
	badgeLockedUrl: string;
	displayOrder: number;
	numAwarded: number;
	numAwardedHardcore: number;
	author: string;
	dateCreated: string;
	dateModified: string;
	type: string;
}

export interface RaGameMetadata {
	id: number;
	title: string;
	consoleId: number;
	consoleName: string;
	imageIconUrl: string;
	numAchievements: number;
	points: number;
	dateModified: string;
	developer?: string;
	publisher?: string;
	genre?: string;
	released?: string;
	numDistinctPlayers?: number;
	imageTitleUrl?: string;
	imageIngameUrl?: string;
	imageBoxArtUrl?: string;
	achievements?: RaAchievement[];
}

export interface RaConsole {
	id: number;
	name: string;
}

export const RA_CONSOLES: RaConsole[] = [
	{ id: 5, name: 'Game Boy Advance' },
	{ id: 4, name: 'Game Boy' },
	{ id: 6, name: 'Game Boy Color' },
	{ id: 3, name: 'SNES/Super Famicom' },
	{ id: 7, name: 'NES/Famicom' },
	{ id: 1, name: 'Genesis/Mega Drive' },
	{ id: 11, name: 'Master System' },
	{ id: 2, name: 'Nintendo 64' },
	{ id: 12, name: 'PlayStation' },
	{ id: 21, name: 'PlayStation 2' },
	{ id: 18, name: 'Nintendo DS' },
	{ id: 9, name: 'Atari 2600' },
	{ id: 8, name: 'PC Engine/TurboGrafx-16' },
	{ id: 10, name: 'Atari 7800' },
	{ id: 47, name: 'Nintendo GameCube' },
	{ id: 14, name: 'Neo Geo Pocket' },
	{ id: 13, name: 'Atari Lynx' },
	{ id: 17, name: 'Atari Jaguar' }
];

// Raw API response types

export interface RaGameListItem {
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

export interface RaAchievementRaw {
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

export interface RaGameExtended {
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
