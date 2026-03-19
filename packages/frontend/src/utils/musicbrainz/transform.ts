import type {
	MusicBrainzArtistCredit,
	MusicBrainzRecording,
	MusicBrainzArtist,
	MusicBrainzReleaseGroup,
	DisplayMusicBrainzRecording,
	DisplayMusicBrainzArtist,
	DisplayMusicBrainzReleaseGroup
} from '$types/musicbrainz.type';

function formatArtistCredits(credits: MusicBrainzArtistCredit[] | undefined): string {
	if (!credits || credits.length === 0) return 'Unknown Artist';
	return credits.map((c) => c.name + (c.joinphrase || '')).join('');
}

function getCoverArtUrl(releaseGroupId: string): string {
	return `https://coverartarchive.org/release-group/${releaseGroupId}/front-250`;
}

function formatDuration(ms: number | undefined): string | null {
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

export function recordingsToDisplay(recordings: MusicBrainzRecording[]): DisplayMusicBrainzRecording[] {
	return recordings.map((r) => {
		const releaseGroupId = r.releases?.[0]?.['release-group']?.id;
		return {
			id: r.id,
			title: r.title,
			duration: formatDuration(r.length),
			artistCredits: formatArtistCredits(r['artist-credit']),
			disambiguation: r.disambiguation || null,
			coverArtUrl: releaseGroupId ? getCoverArtUrl(releaseGroupId) : null,
			firstReleaseTitle: r.releases?.[0]?.title ?? null
		};
	});
}

export function artistsToDisplay(artists: MusicBrainzArtist[]): DisplayMusicBrainzArtist[] {
	return artists.map((a) => ({
		id: a.id,
		name: a.name,
		type: a.type || null,
		country: a.country || null,
		disambiguation: a.disambiguation || null,
		beginYear: a['life-span']?.begin?.split('-')[0] || null,
		endYear: a['life-span']?.end?.split('-')[0] || null,
		tags: (a.tags || []).sort((x, y) => y.count - x.count).map((t) => t.name)
	}));
}

export function releaseGroupsToDisplay(
	releaseGroups: MusicBrainzReleaseGroup[]
): DisplayMusicBrainzReleaseGroup[] {
	return releaseGroups.map((rg) => ({
		id: rg.id,
		title: rg.title,
		primaryType: rg['primary-type'] || null,
		firstReleaseYear: extractYear(rg['first-release-date']),
		artistCredits: formatArtistCredits(rg['artist-credit']),
		coverArtUrl: getCoverArtUrl(rg.id)
	}));
}
