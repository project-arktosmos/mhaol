import type {
	MusicBrainzArtistCredit,
	MusicBrainzReleaseGroup,
	MusicBrainzRecording,
	DisplayMusicBrainzReleaseGroup,
	DisplayMusicBrainzRecording
} from '$types/vendor/musicbrainz.type';

function extractYear(dateString: string | undefined): string {
	if (!dateString) return 'Unknown';
	return dateString.split('-')[0] || 'Unknown';
}

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
