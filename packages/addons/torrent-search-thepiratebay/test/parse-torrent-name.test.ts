import { describe, it, expect } from 'vitest';
import { parseTorrentName, extractSeasonEpisode } from '../src/parse-torrent-name';

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

	describe('season/episode extraction (video)', () => {
		it('returns null season/episode for movies', () => {
			const result = parseTorrentName('Movie.Name.2023.1080p.BluRay.x264', 'Movie Name', '2023');
			expect(result.seasonNumber).toBeNull();
			expect(result.episodeNumber).toBeNull();
			expect(result.isCompleteSeries).toBe(false);
		});

		it('returns null season/episode for music', () => {
			const result = parseTorrentName('Artist - Album S01 FLAC', 'Album', '2023', 'Artist');
			expect(result.seasonNumber).toBeNull();
			expect(result.episodeNumber).toBeNull();
			expect(result.isCompleteSeries).toBe(false);
		});

		it('detects S01E01 pattern', () => {
			const result = parseTorrentName('Breaking.Bad.S01E01.1080p.WEB-DL', 'Breaking Bad', '2008');
			expect(result.seasonNumber).toBe(1);
			expect(result.episodeNumber).toBe(1);
			expect(result.isCompleteSeries).toBe(false);
		});

		it('detects S02E15 pattern', () => {
			const result = parseTorrentName('Show.Name.S02E15.720p.HDTV', 'Show Name', '2020');
			expect(result.seasonNumber).toBe(2);
			expect(result.episodeNumber).toBe(15);
		});

		it('detects season pack S01', () => {
			const result = parseTorrentName('Breaking.Bad.S01.1080p.BluRay', 'Breaking Bad', '2008');
			expect(result.seasonNumber).toBe(1);
			expect(result.episodeNumber).toBeNull();
			expect(result.isCompleteSeries).toBe(false);
		});

		it('detects Season 3 spelled out', () => {
			const result = parseTorrentName('Breaking Bad Season 3 1080p', 'Breaking Bad', '2008');
			expect(result.seasonNumber).toBe(3);
			expect(result.episodeNumber).toBeNull();
		});

		it('detects complete series', () => {
			const result = parseTorrentName(
				'Breaking.Bad.Complete.Series.1080p.BluRay',
				'Breaking Bad',
				'2008'
			);
			expect(result.isCompleteSeries).toBe(true);
			expect(result.seasonNumber).toBeNull();
			expect(result.episodeNumber).toBeNull();
		});

		it('detects full series', () => {
			const result = parseTorrentName('Show Name Full Series 720p', 'Show Name', '2020');
			expect(result.isCompleteSeries).toBe(true);
		});

		it('detects all seasons', () => {
			const result = parseTorrentName('Show.Name.All.Seasons.1080p', 'Show Name', '2020');
			expect(result.isCompleteSeries).toBe(true);
		});

		it('detects S01-S05 range as complete', () => {
			const result = parseTorrentName('Breaking.Bad.S01-S05.1080p.BluRay', 'Breaking Bad', '2008');
			expect(result.isCompleteSeries).toBe(true);
		});

		it('detects Season 1 Episode 5 spelled out', () => {
			const result = parseTorrentName('Show Name Season 2 Episode 10 720p', 'Show Name', '2020');
			expect(result.seasonNumber).toBe(2);
			expect(result.episodeNumber).toBe(10);
		});
	});
});

describe('extractSeasonEpisode', () => {
	it('returns defaults for plain movie names', () => {
		const result = extractSeasonEpisode('Movie.2023.1080p.BluRay');
		expect(result).toEqual({ season: null, episode: null, isCompleteSeries: false });
	});

	it('parses S03E12', () => {
		const result = extractSeasonEpisode('Show.S03E12.720p');
		expect(result).toEqual({ season: 3, episode: 12, isCompleteSeries: false });
	});

	it('parses S01 season pack', () => {
		const result = extractSeasonEpisode('Show.S01.COMPLETE.1080p');
		expect(result).toEqual({ season: 1, episode: null, isCompleteSeries: false });
	});

	it('detects complete series keyword', () => {
		const result = extractSeasonEpisode('Show.Complete.Series.1080p');
		expect(result).toEqual({ season: null, episode: null, isCompleteSeries: true });
	});
});
