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

export type WasmStatus = 'yes' | 'experimental' | 'no';

export const CONSOLE_WASM_STATUS: Record<number, WasmStatus> = {
	5: 'yes', // GBA
	4: 'yes', // GB
	6: 'yes', // GBC
	3: 'yes', // SNES
	7: 'yes', // NES
	1: 'yes', // Genesis
	11: 'yes', // Master System
	2: 'yes', // N64
	12: 'yes', // PS1
	21: 'experimental', // PS2
	18: 'yes', // NDS
	9: 'yes', // Atari 2600
	8: 'yes', // PC Engine
	10: 'yes', // Atari 7800
	47: 'no', // GameCube
	14: 'yes', // Neo Geo Pocket
	13: 'yes', // Atari Lynx
	17: 'yes' // Atari Jaguar
};

export const CONSOLE_SEARCH_NAMES: Record<number, string[]> = {
	5: ['GBA', 'Game Boy Advance'],
	4: ['GB', 'Game Boy'],
	6: ['GBC', 'Game Boy Color'],
	3: ['SNES', 'Super Nintendo'],
	7: ['NES', 'Famicom'],
	1: ['Genesis', 'Mega Drive'],
	11: ['SMS', 'Master System'],
	2: ['N64', 'Nintendo 64'],
	12: ['PSX', 'PS1', 'PlayStation'],
	21: ['PS2', 'PlayStation 2'],
	18: ['NDS', 'Nintendo DS'],
	9: ['Atari 2600'],
	8: ['PC Engine', 'TurboGrafx'],
	10: ['Atari 7800'],
	47: ['GameCube', 'GCN'],
	14: ['Neo Geo Pocket', 'NGP'],
	13: ['Atari Lynx', 'Lynx'],
	17: ['Atari Jaguar', 'Jaguar']
};

export const CONSOLE_EJS_CORE: Record<number, string> = {
	5: 'gba',
	4: 'gb',
	6: 'gb',
	3: 'snes',
	7: 'nes',
	1: 'segaMD',
	11: 'segaMS',
	2: 'n64',
	12: 'psx',
	21: 'ps2',
	18: 'nds',
	9: 'atari2600',
	8: 'pce',
	10: 'atari7800',
	14: 'ngp',
	13: 'lynx',
	17: 'jaguar'
};

export const CONSOLE_ROM_EXTENSIONS: Record<number, string[]> = {
	5: ['gba'],
	4: ['gb'],
	6: ['gbc'],
	3: ['smc', 'sfc', 'zip'],
	7: ['nes', 'zip'],
	1: ['md', 'gen', 'bin', 'zip'],
	11: ['sms', 'zip'],
	2: ['n64', 'z64', 'v64', 'zip'],
	12: ['bin', 'cue', 'iso', 'img', 'zip'],
	21: ['iso'],
	18: ['nds', 'zip'],
	9: ['a26', 'bin', 'zip'],
	8: ['pce', 'zip'],
	10: ['a78', 'zip'],
	14: ['ngp', 'ngc', 'zip'],
	13: ['lnx', 'zip'],
	17: ['j64', 'jag', 'zip']
};

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
