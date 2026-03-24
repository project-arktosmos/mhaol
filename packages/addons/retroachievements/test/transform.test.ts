/* eslint-disable @typescript-eslint/no-explicit-any */
import { describe, it, expect, afterEach } from 'vitest';
import {
	raImageUrl,
	setRaImageBaseUrl,
	gameListItemToDisplay,
	gameExtendedToDisplay
} from '../src/transform';

describe('raImageUrl', () => {
	it('returns full URL for a valid path', () => {
		expect(raImageUrl('/Images/000001.png')).toBe(
			'https://media.retroachievements.org/Images/000001.png'
		);
	});

	it('returns empty string for null', () => {
		expect(raImageUrl(null)).toBe('');
	});

	it('returns empty string for undefined', () => {
		expect(raImageUrl(undefined)).toBe('');
	});

	it('returns empty string for empty string', () => {
		expect(raImageUrl('')).toBe('');
	});

	it('prepends base URL to any non-empty path', () => {
		expect(raImageUrl('/Badge/12345.png')).toBe(
			'https://media.retroachievements.org/Badge/12345.png'
		);
	});
});

describe('setRaImageBaseUrl', () => {
	afterEach(() => {
		setRaImageBaseUrl('https://media.retroachievements.org');
	});

	it('redirects raImageUrl through custom base URL', () => {
		setRaImageBaseUrl('http://localhost:1530/api/retroachievements/image');
		expect(raImageUrl('/Images/000001.png')).toBe(
			'http://localhost:1530/api/retroachievements/image/Images/000001.png'
		);
	});

	it('redirects badge URLs through custom base URL', () => {
		setRaImageBaseUrl('http://localhost:1530/api/retroachievements/image');
		const detail = {
			ID: 1,
			Title: 'Test',
			ConsoleID: 1,
			ConsoleName: 'Console',
			ImageIcon: '/Images/icon.png',
			ImageTitle: null,
			ImageIngame: null,
			ImageBoxArt: null,
			NumAchievements: 1,
			Points: 10,
			DateModified: '',
			Developer: null,
			Publisher: null,
			Genre: null,
			Released: null,
			NumDistinctPlayers: 0,
			Achievements: {
				'1': {
					ID: 1,
					Title: 'Badge Test',
					Description: 'Test',
					Points: 10,
					TrueRatio: 20,
					BadgeName: 'badge_abc',
					DisplayOrder: 0,
					NumAwarded: 100,
					NumAwardedHardcore: 50,
					Author: 'Tester',
					DateCreated: '2024-01-01',
					DateModified: '2024-01-01',
					type: 'progression'
				}
			}
		};

		const result = gameExtendedToDisplay(detail);
		const ach = result.achievements![0];
		expect(ach.badgeUrl).toBe(
			'http://localhost:1530/api/retroachievements/image/Badge/badge_abc.png'
		);
		expect(ach.badgeLockedUrl).toBe(
			'http://localhost:1530/api/retroachievements/image/Badge/badge_abc_lock.png'
		);
	});
});

describe('gameListItemToDisplay', () => {
	it('transforms a complete game list item', () => {
		const item = {
			ID: 1234,
			Title: 'Super Mario World',
			ConsoleID: 3,
			ConsoleName: 'SNES/Super Famicom',
			ImageIcon: '/Images/icon.png',
			ImageTitle: '/Images/title.png',
			ImageIngame: '/Images/ingame.png',
			ImageBoxArt: '/Images/boxart.png',
			NumAchievements: 96,
			Points: 1200,
			DateModified: '2024-01-15'
		};

		const result = gameListItemToDisplay(item);

		expect(result.id).toBe(1234);
		expect(result.title).toBe('Super Mario World');
		expect(result.consoleId).toBe(3);
		expect(result.consoleName).toBe('SNES/Super Famicom');
		expect(result.imageIconUrl).toBe('https://media.retroachievements.org/Images/icon.png');
		expect(result.imageTitleUrl).toBe('https://media.retroachievements.org/Images/title.png');
		expect(result.imageIngameUrl).toBe('https://media.retroachievements.org/Images/ingame.png');
		expect(result.imageBoxArtUrl).toBe('https://media.retroachievements.org/Images/boxart.png');
		expect(result.numAchievements).toBe(96);
		expect(result.points).toBe(1200);
		expect(result.dateModified).toBe('2024-01-15');
	});

	it('handles null image fields', () => {
		const item = {
			ID: 1,
			Title: 'Test Game',
			ConsoleID: 1,
			ConsoleName: 'Genesis/Mega Drive',
			ImageIcon: '/Images/icon.png',
			ImageTitle: null,
			ImageIngame: null,
			ImageBoxArt: null,
			NumAchievements: 10,
			Points: 100,
			DateModified: '2024-01-01'
		};

		const result = gameListItemToDisplay(item);

		expect(result.imageTitleUrl).toBe('');
		expect(result.imageIngameUrl).toBe('');
		expect(result.imageBoxArtUrl).toBe('');
	});

	it('handles missing optional fields with defaults', () => {
		const item = {
			ID: 5,
			Title: 'Minimal Game',
			ConsoleID: 7,
			ConsoleName: null,
			ImageIcon: '/Images/icon.png',
			ImageTitle: null,
			ImageIngame: null,
			ImageBoxArt: null,
			NumAchievements: null,
			Points: null,
			DateModified: null
		};

		const result = gameListItemToDisplay(item as any);

		expect(result.consoleName).toBe('');
		expect(result.numAchievements).toBe(0);
		expect(result.points).toBe(0);
		expect(result.dateModified).toBe('');
	});
});

describe('gameExtendedToDisplay', () => {
	it('transforms a complete game extended detail', () => {
		const detail = {
			ID: 2000,
			Title: 'Zelda',
			ConsoleID: 3,
			ConsoleName: 'SNES/Super Famicom',
			ImageIcon: '/Images/zelda-icon.png',
			ImageTitle: '/Images/zelda-title.png',
			ImageIngame: '/Images/zelda-ingame.png',
			ImageBoxArt: '/Images/zelda-box.png',
			NumAchievements: 50,
			Points: 800,
			DateModified: '2023-06-01',
			Developer: 'Nintendo',
			Publisher: 'Nintendo',
			Genre: 'Action RPG',
			Released: '1991-11-21',
			NumDistinctPlayers: 5000,
			Achievements: {
				'1': {
					ID: 101,
					Title: 'First Sword',
					Description: 'Get the first sword',
					Points: 5,
					TrueRatio: 10,
					BadgeName: 'badge101',
					DisplayOrder: 1,
					NumAwarded: 4000,
					NumAwardedHardcore: 2000,
					Author: 'Creator',
					DateCreated: '2020-01-01',
					DateModified: '2020-06-01',
					type: 'progression'
				},
				'2': {
					ID: 102,
					Title: 'Boss Defeated',
					Description: 'Defeat the boss',
					Points: 25,
					TrueRatio: 50,
					BadgeName: 'badge102',
					DisplayOrder: 2,
					NumAwarded: 2000,
					NumAwardedHardcore: 1000,
					Author: 'Creator',
					DateCreated: '2020-01-01',
					DateModified: '2020-06-01',
					type: 'win_condition'
				}
			}
		};

		const result = gameExtendedToDisplay(detail);

		expect(result.id).toBe(2000);
		expect(result.title).toBe('Zelda');
		expect(result.consoleId).toBe(3);
		expect(result.consoleName).toBe('SNES/Super Famicom');
		expect(result.developer).toBe('Nintendo');
		expect(result.publisher).toBe('Nintendo');
		expect(result.genre).toBe('Action RPG');
		expect(result.released).toBe('1991-11-21');
		expect(result.numDistinctPlayers).toBe(5000);
		expect(result.achievements).toHaveLength(2);
	});

	it('sorts achievements by display order', () => {
		const detail = {
			ID: 1,
			Title: 'Test',
			ConsoleID: 1,
			ConsoleName: 'Test Console',
			ImageIcon: '/icon.png',
			ImageTitle: null,
			ImageIngame: null,
			ImageBoxArt: null,
			NumAchievements: 3,
			Points: 100,
			DateModified: '2024-01-01',
			Developer: null,
			Publisher: null,
			Genre: null,
			Released: null,
			NumDistinctPlayers: 0,
			Achievements: {
				'3': {
					ID: 3,
					Title: 'Third',
					Description: '',
					Points: 10,
					TrueRatio: 20,
					BadgeName: 'b3',
					DisplayOrder: 3,
					NumAwarded: 0,
					NumAwardedHardcore: 0,
					Author: '',
					DateCreated: '',
					DateModified: '',
					type: ''
				},
				'1': {
					ID: 1,
					Title: 'First',
					Description: '',
					Points: 5,
					TrueRatio: 10,
					BadgeName: 'b1',
					DisplayOrder: 1,
					NumAwarded: 0,
					NumAwardedHardcore: 0,
					Author: '',
					DateCreated: '',
					DateModified: '',
					type: ''
				},
				'2': {
					ID: 2,
					Title: 'Second',
					Description: '',
					Points: 15,
					TrueRatio: 30,
					BadgeName: 'b2',
					DisplayOrder: 2,
					NumAwarded: 0,
					NumAwardedHardcore: 0,
					Author: '',
					DateCreated: '',
					DateModified: '',
					type: ''
				}
			}
		};

		const result = gameExtendedToDisplay(detail);

		expect(result.achievements).toHaveLength(3);
		expect(result.achievements![0].title).toBe('First');
		expect(result.achievements![1].title).toBe('Second');
		expect(result.achievements![2].title).toBe('Third');
	});

	it('generates correct badge URLs', () => {
		const detail = {
			ID: 1,
			Title: 'Test',
			ConsoleID: 1,
			ConsoleName: 'Console',
			ImageIcon: '/icon.png',
			ImageTitle: null,
			ImageIngame: null,
			ImageBoxArt: null,
			NumAchievements: 1,
			Points: 10,
			DateModified: '',
			Developer: null,
			Publisher: null,
			Genre: null,
			Released: null,
			NumDistinctPlayers: 0,
			Achievements: {
				'1': {
					ID: 1,
					Title: 'Badge Test',
					Description: 'Test',
					Points: 10,
					TrueRatio: 20,
					BadgeName: 'badge_abc',
					DisplayOrder: 0,
					NumAwarded: 100,
					NumAwardedHardcore: 50,
					Author: 'Tester',
					DateCreated: '2024-01-01',
					DateModified: '2024-01-01',
					type: 'progression'
				}
			}
		};

		const result = gameExtendedToDisplay(detail);
		const ach = result.achievements![0];

		expect(ach.badgeUrl).toBe('https://media.retroachievements.org/Badge/badge_abc.png');
		expect(ach.badgeLockedUrl).toBe('https://media.retroachievements.org/Badge/badge_abc_lock.png');
	});

	it('handles empty BadgeName', () => {
		const detail = {
			ID: 1,
			Title: 'Test',
			ConsoleID: 1,
			ConsoleName: 'Console',
			ImageIcon: '/icon.png',
			ImageTitle: null,
			ImageIngame: null,
			ImageBoxArt: null,
			NumAchievements: 1,
			Points: 10,
			DateModified: '',
			Developer: null,
			Publisher: null,
			Genre: null,
			Released: null,
			NumDistinctPlayers: 0,
			Achievements: {
				'1': {
					ID: 1,
					Title: 'No Badge',
					Description: '',
					Points: 5,
					TrueRatio: 10,
					BadgeName: '',
					DisplayOrder: 0,
					NumAwarded: 0,
					NumAwardedHardcore: 0,
					Author: '',
					DateCreated: '',
					DateModified: '',
					type: ''
				}
			}
		};

		const result = gameExtendedToDisplay(detail);
		expect(result.achievements![0].badgeUrl).toBe('');
		expect(result.achievements![0].badgeLockedUrl).toBe('');
	});

	it('handles null Achievements', () => {
		const detail = {
			ID: 1,
			Title: 'No Achievements',
			ConsoleID: 1,
			ConsoleName: 'Console',
			ImageIcon: '/icon.png',
			ImageTitle: null,
			ImageIngame: null,
			ImageBoxArt: null,
			NumAchievements: 0,
			Points: 0,
			DateModified: '',
			Developer: null,
			Publisher: null,
			Genre: null,
			Released: null,
			NumDistinctPlayers: 0,
			Achievements: null
		};

		const result = gameExtendedToDisplay(detail);
		expect(result.achievements).toEqual([]);
	});

	it('handles missing optional metadata fields', () => {
		const detail = {
			ID: 1,
			Title: 'Basic',
			ConsoleID: 1,
			ConsoleName: null,
			ImageIcon: '/icon.png',
			ImageTitle: null,
			ImageIngame: null,
			ImageBoxArt: null,
			NumAchievements: null,
			Points: null,
			DateModified: null,
			Developer: null,
			Publisher: null,
			Genre: null,
			Released: null,
			NumDistinctPlayers: 0,
			Achievements: null
		};

		const result = gameExtendedToDisplay(detail as any);

		expect(result.consoleName).toBe('');
		expect(result.numAchievements).toBe(0);
		expect(result.points).toBe(0);
		expect(result.dateModified).toBe('');
		expect(result.developer).toBeUndefined();
		expect(result.publisher).toBeUndefined();
		expect(result.genre).toBeUndefined();
		expect(result.released).toBeUndefined();
	});

	it('handles missing achievement optional fields with defaults', () => {
		const detail = {
			ID: 1,
			Title: 'Test',
			ConsoleID: 1,
			ConsoleName: 'Console',
			ImageIcon: '/icon.png',
			ImageTitle: null,
			ImageIngame: null,
			ImageBoxArt: null,
			NumAchievements: 1,
			Points: 10,
			DateModified: '',
			Developer: null,
			Publisher: null,
			Genre: null,
			Released: null,
			NumDistinctPlayers: 0,
			Achievements: {
				'1': {
					ID: 1,
					Title: null,
					Description: null,
					Points: null,
					TrueRatio: null,
					BadgeName: null,
					DisplayOrder: null,
					NumAwarded: null,
					NumAwardedHardcore: null,
					Author: null,
					DateCreated: null,
					DateModified: null,
					type: null
				}
			}
		};

		const result = gameExtendedToDisplay(detail as any);
		const ach = result.achievements![0];

		expect(ach.title).toBe('');
		expect(ach.description).toBe('');
		expect(ach.points).toBe(0);
		expect(ach.trueRatio).toBe(0);
		expect(ach.displayOrder).toBe(0);
		expect(ach.numAwarded).toBe(0);
		expect(ach.numAwardedHardcore).toBe(0);
		expect(ach.author).toBe('');
		expect(ach.dateCreated).toBe('');
		expect(ach.dateModified).toBe('');
		expect(ach.type).toBe('');
	});
});
