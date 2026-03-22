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

/** Console definitions for RetroAchievements browsing */
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
	{ id: 17, name: 'Atari Jaguar' },
];
