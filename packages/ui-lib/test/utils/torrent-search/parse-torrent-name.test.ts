import { describe, it, expect } from 'vitest';
import { parseTorrentName } from '../../../src/utils/torrent-search/parse-torrent-name';

describe('parseTorrentName', () => {
	describe('video quality extraction', () => {
		it('detects 1080p BluRay', () => {
			const result = parseTorrentName('Movie.Name.2023.1080p.BluRay.x264', 'Movie Name', '2023');
			expect(result.quality).toBe('1080p BluRay');
		});

		it('detects 720p WEB-DL', () => {
			const result = parseTorrentName('Movie.720p.WEB-DL', 'Movie', '2023');
			expect(result.quality).toContain('720p');
		});

		it('detects 2160p/4K', () => {
			const result = parseTorrentName('Movie.2160p.HDRip', 'Movie', '2023');
			expect(result.quality).toContain('2160p');
		});

		it('returns Unknown for no quality info', () => {
			const result = parseTorrentName('Movie Name', 'Movie Name', '2023');
			expect(result.quality).toBe('Unknown');
		});

		it('detects source only when no resolution', () => {
			const result = parseTorrentName('Movie.BluRay.x264', 'Movie', '2023');
			expect(result.quality).toBe('BluRay');
		});
	});

	describe('audio quality extraction (music)', () => {
		it('detects FLAC', () => {
			const result = parseTorrentName('Artist - Album (2023) FLAC', 'Album', '2023', 'Artist');
			expect(result.quality).toBe('FLAC');
		});

		it('detects 320kbps MP3', () => {
			const result = parseTorrentName('Artist - Album 320kbps MP3', 'Album', '2023', 'Artist');
			expect(result.quality).toContain('320kbps');
		});

		it('returns Unknown for no audio quality info', () => {
			const result = parseTorrentName('Artist - Album', 'Album', '2023', 'Artist');
			expect(result.quality).toBe('Unknown');
		});
	});

	describe('language extraction', () => {
		it('detects multi-language', () => {
			const result = parseTorrentName('Movie.Multi.1080p', 'Movie', '2023');
			expect(result.languages).toContain('Multi');
		});

		it('detects dual audio', () => {
			const result = parseTorrentName('Movie.Dual.Audio.720p', 'Movie', '2023');
			expect(result.languages).toContain('Dual Audio');
		});

		it('defaults to English when no language detected', () => {
			const result = parseTorrentName('Movie.1080p', 'Movie', '2023');
			expect(result.languages).toBe('English');
		});
	});

	describe('subtitle extraction', () => {
		it('detects subtitles', () => {
			const result = parseTorrentName('Movie.1080p.subs', 'Movie', '2023');
			expect(result.subs).toBe('Yes');
		});

		it('detects hardcoded subs', () => {
			const result = parseTorrentName('Movie.1080p.HC', 'Movie', '2023');
			expect(result.subs).toBe('Hardcoded');
		});

		it('returns none when no subs detected', () => {
			const result = parseTorrentName('Movie.1080p.BluRay', 'Movie', '2023');
			expect(result.subs).toBe('none');
		});

		it('skips subs for music', () => {
			const result = parseTorrentName('Artist - Album subs', 'Album', '2023', 'Artist');
			expect(result.subs).toBe('none');
		});
	});

	describe('relevance scoring (video)', () => {
		it('scores high for exact title and year match', () => {
			const result = parseTorrentName('The Movie (2023) 1080p', 'The Movie', '2023');
			expect(result.relevance).toBeGreaterThanOrEqual(85);
			expect(result.reason).toContain('title matches');
		});

		it('scores zero for year mismatch', () => {
			const result = parseTorrentName('The Movie (2020) 1080p', 'The Movie', '2023');
			expect(result.relevance).toBe(0);
			expect(result.reason).toContain('year mismatch');
		});

		it('gives partial score for partial title match', () => {
			const result = parseTorrentName('The Movie Extended Cut (2023)', 'The Movie', '2023');
			expect(result.relevance).toBeGreaterThan(0);
		});
	});

	describe('relevance scoring (music)', () => {
		it('scores high for exact artist and album match', () => {
			const result = parseTorrentName(
				'Pink Floyd - Dark Side Of The Moon (1973) FLAC',
				'Dark Side Of The Moon',
				'1973',
				'Pink Floyd'
			);
			expect(result.relevance).toBeGreaterThanOrEqual(80);
		});

		it('scores zero for year mismatch in music', () => {
			const result = parseTorrentName('Artist - Album (2020) FLAC', 'Album', '2023', 'Artist');
			expect(result.relevance).toBe(0);
		});
	});
});
