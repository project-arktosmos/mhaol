import type {
	MusicBrainzArtist,
	MusicBrainzArtistCredit,
	MusicBrainzReleaseGroup,
	MusicBrainzRelease,
	MusicBrainzTrack,
	MusicBrainzRecording,
	DisplayMusicBrainzArtist,
	DisplayMusicBrainzArtistDetails,
	DisplayMusicBrainzReleaseGroup,
	DisplayMusicBrainzRelease,
	DisplayMusicBrainzTrack,
	DisplayMusicBrainzRecording
} from './types.js';
import { extractYear } from 'common/utils';

export { extractYear };

// Utility helpers

export function formatArtistCredits(credits: MusicBrainzArtistCredit[] | undefined): string {
	if (!credits || credits.length === 0) return 'Unknown Artist';
	return credits.map((c) => c.name + (c.joinphrase || '')).join('');
}

export function getCoverArtUrl(releaseGroupId: string, size: 250 | 500 = 250): string {
	return `https://coverartarchive.org/release-group/${releaseGroupId}/front-${size}`;
}

export function formatDuration(ms: number | undefined): string | null {
	if (!ms) return null;
	const totalSeconds = Math.floor(ms / 1000);
	const minutes = Math.floor(totalSeconds / 60);
	const seconds = totalSeconds % 60;
	return `${minutes}:${seconds.toString().padStart(2, '0')}`;
}

// Artist transforms

export function artistToDisplay(artist: MusicBrainzArtist): DisplayMusicBrainzArtist {
	const tags = (artist.tags || [])
		.sort((a, b) => b.count - a.count)
		.slice(0, 5)
		.map((t) => t.name);

	return {
		id: artist.id,
		name: artist.name,
		sortName: artist['sort-name'],
		type: artist.type || null,
		country: artist.country || null,
		disambiguation: artist.disambiguation || null,
		beginYear:
			extractYear(artist['life-span']?.begin) !== 'Unknown'
				? extractYear(artist['life-span']?.begin)
				: null,
		endYear: artist['life-span']?.end ? extractYear(artist['life-span']?.end) : null,
		ended: artist['life-span']?.ended || false,
		tags,
		score: artist.score || 0
	};
}

export function artistsToDisplay(artists: MusicBrainzArtist[]): DisplayMusicBrainzArtist[] {
	return artists.map((a) => artistToDisplay(a));
}

export function artistDetailsToDisplay(
	artist: MusicBrainzArtist
): DisplayMusicBrainzArtistDetails {
	const releaseGroups = (artist['release-groups'] || [])
		.filter(
			(rg) => rg['primary-type'] && ['Album', 'EP', 'Single'].includes(rg['primary-type'])
		)
		.sort((a, b) => {
			const dateA = a['first-release-date'] || '';
			const dateB = b['first-release-date'] || '';
			return dateB.localeCompare(dateA);
		})
		.map((rg) => releaseGroupToDisplay(rg));

	return {
		...artistToDisplay(artist),
		releaseGroups,
		imageUrl: null
	};
}

// Release group transforms

export function releaseGroupToDisplay(
	rg: MusicBrainzReleaseGroup
): DisplayMusicBrainzReleaseGroup {
	return {
		id: rg.id,
		title: rg.title,
		primaryType: rg['primary-type'] || null,
		secondaryTypes: rg['secondary-types'] || [],
		firstReleaseYear: extractYear(rg['first-release-date']),
		artistCredits: formatArtistCredits(rg['artist-credit']),
		coverArtUrl: getCoverArtUrl(rg.id),
		score: rg.score || 0
	};
}

export function releaseGroupsToDisplay(
	releaseGroups: MusicBrainzReleaseGroup[]
): DisplayMusicBrainzReleaseGroup[] {
	return releaseGroups.map((rg) => releaseGroupToDisplay(rg));
}

// Release & Track transforms

export function trackToDisplay(track: MusicBrainzTrack): DisplayMusicBrainzTrack {
	return {
		id: track.id,
		number: track.number,
		title: track.recording.title,
		duration: formatDuration(track.recording.length || track.length),
		durationMs: track.recording.length || track.length || null,
		artistCredits: formatArtistCredits(track['artist-credit'])
	};
}

export function recordingToDisplay(recording: MusicBrainzRecording): DisplayMusicBrainzRecording {
	const releaseGroupId = recording.releases?.[0]?.['release-group']?.id;
	return {
		id: recording.id,
		title: recording.title,
		duration: formatDuration(recording.length),
		durationMs: recording.length || null,
		artistCredits: formatArtistCredits(recording['artist-credit']),
		disambiguation: recording.disambiguation || null,
		coverArtUrl: releaseGroupId ? getCoverArtUrl(releaseGroupId) : null,
		firstReleaseTitle: recording.releases?.[0]?.title ?? null,
		score: recording.score || 0
	};
}

export function recordingsToDisplay(
	recordings: MusicBrainzRecording[]
): DisplayMusicBrainzRecording[] {
	return recordings.map((r) => recordingToDisplay(r));
}

export function releaseToDisplay(release: MusicBrainzRelease): DisplayMusicBrainzRelease {
	const tracks: DisplayMusicBrainzTrack[] = [];
	for (const media of release.media || []) {
		for (const track of media.tracks || []) {
			tracks.push(trackToDisplay(track));
		}
	}

	const label = release['label-info']?.[0]?.label?.name || null;

	return {
		id: release.id,
		title: release.title,
		date: release.date || null,
		status: release.status || null,
		country: release.country || null,
		artistCredits: formatArtistCredits(release['artist-credit']),
		trackCount: release['track-count'] || tracks.length,
		label,
		tracks
	};
}
