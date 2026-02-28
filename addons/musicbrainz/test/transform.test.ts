import { describe, it, expect } from 'vitest';
import {
	formatArtistCredits,
	getCoverArtUrl,
	extractYear,
	formatDuration,
	artistToDisplay,
	artistsToDisplay,
	artistDetailsToDisplay,
	releaseGroupToDisplay,
	releaseGroupsToDisplay,
	trackToDisplay,
	recordingToDisplay,
	recordingsToDisplay,
	releaseToDisplay
} from '../src/transform.js';
import type {
	MusicBrainzArtist,
	MusicBrainzArtistCredit,
	MusicBrainzReleaseGroup,
	MusicBrainzRelease,
	MusicBrainzTrack,
	MusicBrainzRecording
} from '../src/types.js';

function makeArtist(overrides: Partial<MusicBrainzArtist> = {}): MusicBrainzArtist {
	return {
		id: 'artist-1',
		name: 'Test Artist',
		'sort-name': 'Artist, Test',
		type: 'Person',
		country: 'US',
		disambiguation: 'the band',
		'life-span': { begin: '1990-01-01', ended: false },
		tags: [
			{ count: 10, name: 'rock' },
			{ count: 5, name: 'alternative' }
		],
		score: 100,
		...overrides
	};
}

function makeReleaseGroup(overrides: Partial<MusicBrainzReleaseGroup> = {}): MusicBrainzReleaseGroup {
	return {
		id: 'rg-1',
		title: 'Test Album',
		'primary-type': 'Album',
		'secondary-types': [],
		'first-release-date': '2020-05-15',
		'artist-credit': [{ name: 'Test Artist', joinphrase: '', artist: { id: 'a-1', name: 'Test Artist', 'sort-name': 'Artist, Test' } }],
		score: 90,
		...overrides
	};
}

function makeTrack(overrides: Partial<MusicBrainzTrack> = {}): MusicBrainzTrack {
	return {
		id: 'track-1',
		number: '1',
		title: 'Test Track',
		length: 240000,
		position: 1,
		'artist-credit': [{ name: 'Test Artist', joinphrase: '', artist: { id: 'a-1', name: 'Test Artist', 'sort-name': 'Artist, Test' } }],
		recording: {
			id: 'rec-1',
			title: 'Test Track',
			length: 240000
		},
		...overrides
	};
}

function makeRecording(overrides: Partial<MusicBrainzRecording> = {}): MusicBrainzRecording {
	return {
		id: 'rec-1',
		title: 'Test Recording',
		length: 180000,
		disambiguation: 'live version',
		'artist-credit': [{ name: 'Test Artist', joinphrase: '', artist: { id: 'a-1', name: 'Test Artist', 'sort-name': 'Artist, Test' } }],
		score: 85,
		...overrides
	};
}

// Utility helpers

describe('formatArtistCredits', () => {
	it('formats single artist', () => {
		const credits: MusicBrainzArtistCredit[] = [
			{ name: 'Radiohead', joinphrase: '', artist: { id: '1', name: 'Radiohead', 'sort-name': 'Radiohead' } }
		];
		expect(formatArtistCredits(credits)).toBe('Radiohead');
	});

	it('formats multiple artists with join phrases', () => {
		const credits: MusicBrainzArtistCredit[] = [
			{ name: 'Artist A', joinphrase: ' feat. ', artist: { id: '1', name: 'Artist A', 'sort-name': 'A' } },
			{ name: 'Artist B', joinphrase: '', artist: { id: '2', name: 'Artist B', 'sort-name': 'B' } }
		];
		expect(formatArtistCredits(credits)).toBe('Artist A feat. Artist B');
	});

	it('returns Unknown Artist for undefined', () => {
		expect(formatArtistCredits(undefined)).toBe('Unknown Artist');
	});

	it('returns Unknown Artist for empty array', () => {
		expect(formatArtistCredits([])).toBe('Unknown Artist');
	});
});

describe('getCoverArtUrl', () => {
	it('returns URL with default size', () => {
		expect(getCoverArtUrl('rg-123')).toBe(
			'https://coverartarchive.org/release-group/rg-123/front-250'
		);
	});

	it('returns URL with custom size', () => {
		expect(getCoverArtUrl('rg-123', 500)).toBe(
			'https://coverartarchive.org/release-group/rg-123/front-500'
		);
	});
});

describe('extractYear', () => {
	it('extracts year from date string', () => {
		expect(extractYear('2020-05-15')).toBe('2020');
	});

	it('returns Unknown for undefined', () => {
		expect(extractYear(undefined)).toBe('Unknown');
	});

	it('returns Unknown for empty string', () => {
		expect(extractYear('')).toBe('Unknown');
	});
});

describe('formatDuration', () => {
	it('formats milliseconds as MM:SS', () => {
		expect(formatDuration(240000)).toBe('4:00');
	});

	it('pads seconds with zero', () => {
		expect(formatDuration(185000)).toBe('3:05');
	});

	it('returns null for undefined', () => {
		expect(formatDuration(undefined)).toBeNull();
	});

	it('returns null for 0', () => {
		expect(formatDuration(0)).toBeNull();
	});
});

// Artist transforms

describe('artistToDisplay', () => {
	it('maps all fields correctly', () => {
		const artist = makeArtist();
		const display = artistToDisplay(artist);

		expect(display.id).toBe('artist-1');
		expect(display.name).toBe('Test Artist');
		expect(display.sortName).toBe('Artist, Test');
		expect(display.type).toBe('Person');
		expect(display.country).toBe('US');
		expect(display.disambiguation).toBe('the band');
		expect(display.beginYear).toBe('1990');
		expect(display.endYear).toBeNull();
		expect(display.ended).toBe(false);
		expect(display.tags).toEqual(['rock', 'alternative']);
		expect(display.score).toBe(100);
	});

	it('sorts tags by count and limits to 5', () => {
		const tags = Array.from({ length: 8 }, (_, i) => ({ count: i, name: `tag-${i}` }));
		const artist = makeArtist({ tags });
		const display = artistToDisplay(artist);

		expect(display.tags).toHaveLength(5);
		expect(display.tags[0]).toBe('tag-7');
	});

	it('handles missing optional fields', () => {
		const artist = makeArtist({
			type: undefined,
			country: undefined,
			disambiguation: undefined,
			'life-span': undefined,
			tags: undefined,
			score: undefined
		});
		const display = artistToDisplay(artist);

		expect(display.type).toBeNull();
		expect(display.country).toBeNull();
		expect(display.disambiguation).toBeNull();
		expect(display.beginYear).toBeNull();
		expect(display.endYear).toBeNull();
		expect(display.ended).toBe(false);
		expect(display.tags).toEqual([]);
		expect(display.score).toBe(0);
	});

	it('maps end year when artist has ended', () => {
		const artist = makeArtist({
			'life-span': { begin: '1985-01-01', end: '2005-12-31', ended: true }
		});
		const display = artistToDisplay(artist);

		expect(display.beginYear).toBe('1985');
		expect(display.endYear).toBe('2005');
		expect(display.ended).toBe(true);
	});
});

describe('artistsToDisplay', () => {
	it('maps an array of artists', () => {
		const artists = [makeArtist({ id: 'a-1' }), makeArtist({ id: 'a-2' })];
		const display = artistsToDisplay(artists);
		expect(display).toHaveLength(2);
		expect(display[0].id).toBe('a-1');
		expect(display[1].id).toBe('a-2');
	});
});

describe('artistDetailsToDisplay', () => {
	it('includes filtered and sorted release groups', () => {
		const artist = makeArtist({
			'release-groups': [
				makeReleaseGroup({ id: 'rg-1', 'primary-type': 'Album', 'first-release-date': '2018-01-01' }),
				makeReleaseGroup({ id: 'rg-2', 'primary-type': 'Album', 'first-release-date': '2020-01-01' }),
				makeReleaseGroup({ id: 'rg-3', 'primary-type': 'Compilation', 'first-release-date': '2019-01-01' }),
				makeReleaseGroup({ id: 'rg-4', 'primary-type': 'EP', 'first-release-date': '2021-01-01' })
			]
		});
		const display = artistDetailsToDisplay(artist);

		expect(display.releaseGroups).toHaveLength(3);
		expect(display.releaseGroups[0].id).toBe('rg-4');
		expect(display.releaseGroups[1].id).toBe('rg-2');
		expect(display.releaseGroups[2].id).toBe('rg-1');
		expect(display.imageUrl).toBeNull();
	});

	it('handles missing release groups', () => {
		const artist = makeArtist({ 'release-groups': undefined });
		const display = artistDetailsToDisplay(artist);
		expect(display.releaseGroups).toEqual([]);
	});
});

// Release group transforms

describe('releaseGroupToDisplay', () => {
	it('maps all fields correctly', () => {
		const rg = makeReleaseGroup();
		const display = releaseGroupToDisplay(rg);

		expect(display.id).toBe('rg-1');
		expect(display.title).toBe('Test Album');
		expect(display.primaryType).toBe('Album');
		expect(display.secondaryTypes).toEqual([]);
		expect(display.firstReleaseYear).toBe('2020');
		expect(display.artistCredits).toBe('Test Artist');
		expect(display.coverArtUrl).toBe('https://coverartarchive.org/release-group/rg-1/front-250');
		expect(display.score).toBe(90);
	});

	it('handles missing optional fields', () => {
		const rg = makeReleaseGroup({
			'primary-type': undefined,
			'secondary-types': undefined,
			'first-release-date': undefined,
			'artist-credit': undefined,
			score: undefined
		});
		const display = releaseGroupToDisplay(rg);

		expect(display.primaryType).toBeNull();
		expect(display.secondaryTypes).toEqual([]);
		expect(display.firstReleaseYear).toBe('Unknown');
		expect(display.artistCredits).toBe('Unknown Artist');
		expect(display.score).toBe(0);
	});
});

describe('releaseGroupsToDisplay', () => {
	it('maps an array of release groups', () => {
		const rgs = [makeReleaseGroup({ id: 'rg-1' }), makeReleaseGroup({ id: 'rg-2' })];
		const display = releaseGroupsToDisplay(rgs);
		expect(display).toHaveLength(2);
		expect(display[0].id).toBe('rg-1');
		expect(display[1].id).toBe('rg-2');
	});
});

// Track transforms

describe('trackToDisplay', () => {
	it('maps all fields correctly', () => {
		const track = makeTrack();
		const display = trackToDisplay(track);

		expect(display.id).toBe('track-1');
		expect(display.number).toBe('1');
		expect(display.title).toBe('Test Track');
		expect(display.duration).toBe('4:00');
		expect(display.durationMs).toBe(240000);
		expect(display.artistCredits).toBe('Test Artist');
	});

	it('uses track length as fallback when recording length is missing', () => {
		const track = makeTrack({
			length: 300000,
			recording: { id: 'rec-1', title: 'Test', length: undefined }
		});
		const display = trackToDisplay(track);
		expect(display.duration).toBe('5:00');
		expect(display.durationMs).toBe(300000);
	});

	it('handles missing length', () => {
		const track = makeTrack({
			length: undefined,
			recording: { id: 'rec-1', title: 'Test', length: undefined }
		});
		const display = trackToDisplay(track);
		expect(display.duration).toBeNull();
		expect(display.durationMs).toBeNull();
	});
});

// Recording transforms

describe('recordingToDisplay', () => {
	it('maps all fields correctly', () => {
		const recording = makeRecording({
			releases: [{
				id: 'rel-1',
				title: 'Album',
				'release-group': { id: 'rg-1', title: 'Album' }
			}]
		});
		const display = recordingToDisplay(recording);

		expect(display.id).toBe('rec-1');
		expect(display.title).toBe('Test Recording');
		expect(display.duration).toBe('3:00');
		expect(display.durationMs).toBe(180000);
		expect(display.artistCredits).toBe('Test Artist');
		expect(display.disambiguation).toBe('live version');
		expect(display.coverArtUrl).toBe('https://coverartarchive.org/release-group/rg-1/front-250');
		expect(display.score).toBe(85);
	});

	it('handles missing releases', () => {
		const recording = makeRecording({ releases: undefined });
		const display = recordingToDisplay(recording);
		expect(display.coverArtUrl).toBeNull();
	});

	it('handles missing disambiguation', () => {
		const recording = makeRecording({ disambiguation: undefined });
		const display = recordingToDisplay(recording);
		expect(display.disambiguation).toBeNull();
	});
});

describe('recordingsToDisplay', () => {
	it('maps an array of recordings', () => {
		const recordings = [makeRecording({ id: 'r-1' }), makeRecording({ id: 'r-2' })];
		const display = recordingsToDisplay(recordings);
		expect(display).toHaveLength(2);
		expect(display[0].id).toBe('r-1');
		expect(display[1].id).toBe('r-2');
	});
});

// Release transforms

describe('releaseToDisplay', () => {
	it('maps all fields and flattens tracks from media', () => {
		const release: MusicBrainzRelease = {
			id: 'rel-1',
			title: 'Test Release',
			date: '2020-05-15',
			status: 'Official',
			country: 'US',
			'track-count': 2,
			'label-info': [{ label: { id: 'l-1', name: 'Test Label' } }],
			'artist-credit': [{ name: 'Test Artist', joinphrase: '', artist: { id: 'a-1', name: 'Test Artist', 'sort-name': 'Artist, Test' } }],
			media: [
				{
					position: 1,
					'track-count': 2,
					tracks: [makeTrack({ id: 't-1', number: '1' }), makeTrack({ id: 't-2', number: '2' })]
				}
			]
		};
		const display = releaseToDisplay(release);

		expect(display.id).toBe('rel-1');
		expect(display.title).toBe('Test Release');
		expect(display.date).toBe('2020-05-15');
		expect(display.status).toBe('Official');
		expect(display.country).toBe('US');
		expect(display.artistCredits).toBe('Test Artist');
		expect(display.trackCount).toBe(2);
		expect(display.label).toBe('Test Label');
		expect(display.tracks).toHaveLength(2);
	});

	it('handles missing optional fields', () => {
		const release: MusicBrainzRelease = {
			id: 'rel-1',
			title: 'Test Release'
		};
		const display = releaseToDisplay(release);

		expect(display.date).toBeNull();
		expect(display.status).toBeNull();
		expect(display.country).toBeNull();
		expect(display.artistCredits).toBe('Unknown Artist');
		expect(display.trackCount).toBe(0);
		expect(display.label).toBeNull();
		expect(display.tracks).toEqual([]);
	});

	it('uses track count from tracks when track-count is missing', () => {
		const release: MusicBrainzRelease = {
			id: 'rel-1',
			title: 'Test',
			media: [{ position: 1, 'track-count': 1, tracks: [makeTrack()] }]
		};
		const display = releaseToDisplay(release);
		expect(display.trackCount).toBe(1);
	});
});
