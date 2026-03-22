import { describe, it, expect } from 'vitest';
import {
	formatArtistCredits,
	getCoverArtUrl,
	formatDuration,
	recordingToDisplay,
	artistToDisplay,
	releaseGroupToDisplay,
	trackToDisplay,
	releaseToDisplay
} from '../../../src/utils/musicbrainz/transform';

describe('formatArtistCredits', () => {
	it('returns Unknown Artist for undefined', () => {
		expect(formatArtistCredits(undefined)).toBe('Unknown Artist');
	});

	it('returns Unknown Artist for empty array', () => {
		expect(formatArtistCredits([])).toBe('Unknown Artist');
	});

	it('formats single artist', () => {
		expect(formatArtistCredits([{ name: 'Artist', joinphrase: '' }])).toBe('Artist');
	});

	it('formats multiple artists with join phrases', () => {
		const credits = [
			{ name: 'Artist1', joinphrase: ' feat. ' },
			{ name: 'Artist2', joinphrase: '' }
		];
		expect(formatArtistCredits(credits)).toBe('Artist1 feat. Artist2');
	});

	it('handles missing joinphrase', () => {
		const credits = [{ name: 'Solo' }];
		expect(formatArtistCredits(credits as any)).toBe('Solo');
	});
});

describe('getCoverArtUrl', () => {
	it('returns URL with default size', () => {
		expect(getCoverArtUrl('abc-123')).toBe(
			'https://coverartarchive.org/release-group/abc-123/front-250'
		);
	});

	it('returns URL with custom size', () => {
		expect(getCoverArtUrl('abc-123', 500)).toBe(
			'https://coverartarchive.org/release-group/abc-123/front-500'
		);
	});
});

describe('formatDuration', () => {
	it('returns null for undefined', () => {
		expect(formatDuration(undefined)).toBeNull();
	});

	it('returns null for zero', () => {
		expect(formatDuration(0)).toBeNull();
	});

	it('formats milliseconds to m:ss', () => {
		expect(formatDuration(185000)).toBe('3:05');
	});

	it('formats exact minutes', () => {
		expect(formatDuration(180000)).toBe('3:00');
	});

	it('pads seconds with leading zero', () => {
		expect(formatDuration(63000)).toBe('1:03');
	});
});

describe('recordingToDisplay', () => {
	it('transforms a recording', () => {
		const recording = {
			id: 'rec-1',
			title: 'Song Title',
			length: 240000,
			'artist-credit': [{ name: 'Artist', joinphrase: '' }],
			disambiguation: 'live',
			releases: [
				{
					title: 'Album',
					'release-group': { id: 'rg-1' }
				}
			],
			score: 95
		};

		const result = recordingToDisplay(recording as any);
		expect(result.title).toBe('Song Title');
		expect(result.duration).toBe('4:00');
		expect(result.artistCredits).toBe('Artist');
		expect(result.coverArtUrl).toContain('rg-1');
		expect(result.firstReleaseTitle).toBe('Album');
		expect(result.score).toBe(95);
	});

	it('handles recording with no releases', () => {
		const recording = {
			id: 'rec-2',
			title: 'Song',
			'artist-credit': [],
			score: 0
		};

		const result = recordingToDisplay(recording as any);
		expect(result.coverArtUrl).toBeNull();
		expect(result.firstReleaseTitle).toBeNull();
		expect(result.artistCredits).toBe('Unknown Artist');
	});
});

describe('artistToDisplay', () => {
	it('transforms an artist', () => {
		const artist = {
			id: 'art-1',
			name: 'Band Name',
			'sort-name': 'Name, Band',
			type: 'Group',
			country: 'US',
			disambiguation: 'rock band',
			'life-span': { begin: '1990-01-01', end: '2020-12-31', ended: true },
			tags: [
				{ name: 'rock', count: 10 },
				{ name: 'alternative', count: 5 }
			],
			score: 90
		};

		const result = artistToDisplay(artist as any);
		expect(result.name).toBe('Band Name');
		expect(result.type).toBe('Group');
		expect(result.beginYear).toBe('1990');
		expect(result.endYear).toBe('2020');
		expect(result.ended).toBe(true);
		expect(result.tags).toEqual(['rock', 'alternative']);
	});

	it('handles artist with missing optional fields', () => {
		const artist = {
			id: 'art-2',
			name: 'Solo',
			'sort-name': 'Solo'
		};

		const result = artistToDisplay(artist as any);
		expect(result.type).toBeNull();
		expect(result.country).toBeNull();
		expect(result.tags).toEqual([]);
		expect(result.ended).toBe(false);
	});
});

describe('releaseGroupToDisplay', () => {
	it('transforms a release group', () => {
		const rg = {
			id: 'rg-1',
			title: 'Album Title',
			'primary-type': 'Album',
			'secondary-types': ['Compilation'],
			'first-release-date': '2000-05-15',
			'artist-credit': [{ name: 'Artist', joinphrase: '' }],
			score: 85
		};

		const result = releaseGroupToDisplay(rg as any);
		expect(result.title).toBe('Album Title');
		expect(result.primaryType).toBe('Album');
		expect(result.firstReleaseYear).toBe('2000');
		expect(result.coverArtUrl).toContain('rg-1');
	});
});

describe('trackToDisplay', () => {
	it('transforms a track', () => {
		const track = {
			id: 'track-1',
			number: '3',
			title: 'Track Three',
			length: 200000,
			'artist-credit': [{ name: 'Artist', joinphrase: '' }]
		};

		const result = trackToDisplay(track as any);
		expect(result.number).toBe('3');
		expect(result.title).toBe('Track Three');
		expect(result.duration).toBe('3:20');
	});
});

describe('releaseToDisplay', () => {
	it('transforms a release with tracks', () => {
		const release = {
			id: 'rel-1',
			title: 'Album',
			date: '2023-01-01',
			status: 'Official',
			country: 'US',
			'artist-credit': [{ name: 'Artist', joinphrase: '' }],
			'track-count': 10,
			'label-info': [{ label: { name: 'Record Label' } }],
			media: [
				{
					'track-count': 5,
					tracks: [
						{
							id: 't1',
							number: '1',
							title: 'Track 1',
							length: 180000,
							'artist-credit': [{ name: 'Artist', joinphrase: '' }]
						}
					]
				}
			]
		};

		const result = releaseToDisplay(release as any);
		expect(result.title).toBe('Album');
		expect(result.label).toBe('Record Label');
		expect(result.tracks).toHaveLength(1);
		expect(result.trackCount).toBe(5);
	});

	it('handles release with no media', () => {
		const release = {
			id: 'rel-2',
			title: 'Album',
			'artist-credit': [],
			'track-count': 12
		};

		const result = releaseToDisplay(release as any);
		expect(result.tracks).toEqual([]);
		expect(result.trackCount).toBe(12);
		expect(result.label).toBeNull();
	});
});
