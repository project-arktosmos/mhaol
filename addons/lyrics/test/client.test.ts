import { describe, it, expect } from 'vitest';
import { parseLrcToSyncedLines, parseResponse } from '../src/client';
import type { LrcLibResponse } from '../src/types';

describe('parseLrcToSyncedLines', () => {
	it('parses standard LRC format', () => {
		const lrc = '[00:10.50]Welcome to the show\n[00:15.20]This is where it starts';
		const result = parseLrcToSyncedLines(lrc);

		expect(result).toEqual([
			{ time: 10.5, text: 'Welcome to the show' },
			{ time: 15.2, text: 'This is where it starts' }
		]);
	});

	it('handles three-digit milliseconds', () => {
		const lrc = '[01:22.300]Verse one begins';
		const result = parseLrcToSyncedLines(lrc);

		expect(result).toEqual([{ time: 82.3, text: 'Verse one begins' }]);
	});

	it('handles two-digit milliseconds by padding', () => {
		const lrc = '[00:05.50]Short ms';
		const result = parseLrcToSyncedLines(lrc);

		expect(result).toEqual([{ time: 5.5, text: 'Short ms' }]);
	});

	it('returns empty array for invalid input', () => {
		expect(parseLrcToSyncedLines('')).toEqual([]);
		expect(parseLrcToSyncedLines('No timestamps here')).toEqual([]);
	});

	it('handles empty text lines', () => {
		const lrc = '[00:05.00]\n[00:10.00]Some text';
		const result = parseLrcToSyncedLines(lrc);

		expect(result).toEqual([
			{ time: 5, text: '' },
			{ time: 10, text: 'Some text' }
		]);
	});

	it('sorts lines by time', () => {
		const lrc = '[00:20.00]Second\n[00:10.00]First';
		const result = parseLrcToSyncedLines(lrc);

		expect(result[0].text).toBe('First');
		expect(result[1].text).toBe('Second');
	});
});

describe('parseResponse', () => {
	it('maps LrcLibResponse to Lyrics with synced lyrics', () => {
		const response: LrcLibResponse = {
			id: 123,
			trackName: 'Test Song',
			artistName: 'Test Artist',
			albumName: 'Test Album',
			duration: 180,
			instrumental: false,
			plainLyrics: 'Plain text lyrics',
			syncedLyrics: '[00:05.00]Line one\n[00:10.00]Line two'
		};

		const result = parseResponse(response);

		expect(result.id).toBe(123);
		expect(result.trackName).toBe('Test Song');
		expect(result.artistName).toBe('Test Artist');
		expect(result.albumName).toBe('Test Album');
		expect(result.duration).toBe(180);
		expect(result.instrumental).toBe(false);
		expect(result.plainLyrics).toBe('Plain text lyrics');
		expect(result.syncedLyrics).toEqual([
			{ time: 5, text: 'Line one' },
			{ time: 10, text: 'Line two' }
		]);
	});

	it('handles null synced lyrics', () => {
		const response: LrcLibResponse = {
			id: 456,
			trackName: 'No Sync',
			artistName: 'Artist',
			albumName: 'Album',
			duration: 120,
			instrumental: false,
			plainLyrics: 'Just plain',
			syncedLyrics: null
		};

		const result = parseResponse(response);
		expect(result.syncedLyrics).toBeNull();
		expect(result.plainLyrics).toBe('Just plain');
	});

	it('handles instrumental tracks', () => {
		const response: LrcLibResponse = {
			id: 789,
			trackName: 'Instrumental',
			artistName: 'Artist',
			albumName: 'Album',
			duration: 240,
			instrumental: true,
			plainLyrics: null,
			syncedLyrics: null
		};

		const result = parseResponse(response);
		expect(result.instrumental).toBe(true);
		expect(result.plainLyrics).toBeNull();
		expect(result.syncedLyrics).toBeNull();
	});
});
