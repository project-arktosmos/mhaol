import type {
	MusicBrainzArtistCredit,
	MusicBrainzRecording,
	MusicBrainzArtist,
	MusicBrainzReleaseGroup,
	MusicBrainzRelease,
	MusicBrainzTrack,
	DisplayMusicBrainzRecording,
	DisplayMusicBrainzArtist,
	DisplayMusicBrainzReleaseGroup,
	DisplayMusicBrainzRelease,
	DisplayMusicBrainzTrack
} from './types.js';

export function formatArtistCredits(credits: MusicBrainzArtistCredit[] | undefined): string {
	if (!credits || credits.length === 0) return 'Unknown Artist';
	return credits.map((c) => c.name + (c.joinphrase || '')).join('');
}

let coverArtBaseUrl: string | null = null;

/** Set the base URL for MusicBrainz cover art (e.g. to route through local backend cache). */
export function setCoverArtBaseUrl(url: string) {
	coverArtBaseUrl = url;
}

export function getCoverArtUrl(releaseGroupId: string, size: 250 | 500 = 250): string {
	if (coverArtBaseUrl) {
		return `${coverArtBaseUrl}/${releaseGroupId}/${size}`;
	}
	return `https://coverartarchive.org/release-group/${releaseGroupId}/front-${size}`;
}

export function formatDuration(ms: number | undefined): string | null {
	if (!ms) return null;
	const totalSeconds = Math.floor(ms / 1000);
	const minutes = Math.floor(totalSeconds / 60);
	const seconds = totalSeconds % 60;
	return `${minutes}:${seconds.toString().padStart(2, '0')}`;
}

function extractYear(dateString: string | undefined): string {
	if (!dateString) return 'Unknown';
	return dateString.split('-')[0] || 'Unknown';
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
	return recordings.map(recordingToDisplay);
}

export function artistToDisplay(artist: MusicBrainzArtist): DisplayMusicBrainzArtist {
	return {
		id: artist.id,
		name: artist.name,
		sortName: artist['sort-name'],
		type: artist.type || null,
		country: artist.country || null,
		disambiguation: artist.disambiguation || null,
		beginYear: artist['life-span']?.begin?.split('-')[0] || null,
		endYear: artist['life-span']?.end?.split('-')[0] || null,
		ended: artist['life-span']?.ended ?? false,
		tags: (artist.tags || []).sort((x, y) => y.count - x.count).map((t) => t.name),
		score: artist.score || 0
	};
}

export function artistsToDisplay(artists: MusicBrainzArtist[]): DisplayMusicBrainzArtist[] {
	return artists.map(artistToDisplay);
}

export function releaseGroupToDisplay(rg: MusicBrainzReleaseGroup): DisplayMusicBrainzReleaseGroup {
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
	return releaseGroups.map(releaseGroupToDisplay);
}

export function trackToDisplay(track: MusicBrainzTrack): DisplayMusicBrainzTrack {
	return {
		id: track.id,
		number: track.number,
		title: track.title,
		duration: formatDuration(track.length),
		durationMs: track.length || null,
		artistCredits: formatArtistCredits(track['artist-credit'])
	};
}

export function releaseToDisplay(release: MusicBrainzRelease): DisplayMusicBrainzRelease {
	const tracks: DisplayMusicBrainzTrack[] = [];
	let trackCount = 0;
	for (const media of release.media ?? []) {
		trackCount += media['track-count'];
		for (const track of media.tracks ?? []) {
			tracks.push(trackToDisplay(track));
		}
	}
	const labelInfo = release['label-info']?.[0];
	return {
		id: release.id,
		title: release.title,
		date: release.date ?? null,
		status: release.status ?? null,
		country: release.country ?? null,
		artistCredits: formatArtistCredits(release['artist-credit']),
		trackCount: trackCount || release['track-count'] || 0,
		label: labelInfo?.label?.name ?? null,
		tracks
	};
}
