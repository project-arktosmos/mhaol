import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { fetchRecording } from 'musicbrainz';
import { recordingToDisplay } from 'musicbrainz/transform';
import { fetchLrcLibLyrics, parseLrcToSyncedLines } from 'lyrics/client';
import type { Lyrics } from 'lyrics/types';
import type { LrcLibLyricsRow } from 'lyrics/cache-repository';

function rowToLyrics(row: LrcLibLyricsRow): Lyrics {
	return {
		id: row.lrclib_id,
		trackName: row.track_name,
		artistName: row.artist_name,
		albumName: row.album_name,
		duration: row.duration,
		instrumental: row.instrumental === 1,
		plainLyrics: row.plain_lyrics,
		syncedLyrics: row.synced_lyrics ? parseLrcToSyncedLines(row.synced_lyrics) : null
	};
}

export const GET: RequestHandler = async ({ params, locals }) => {
	const itemId = params.itemId;

	// Check lookup table first
	const lookup = locals.lrclibCacheRepo.getLookup(itemId);
	if (lookup) {
		if (lookup.status === 'not_found') {
			return json({
				id: 0,
				trackName: '',
				artistName: '',
				albumName: '',
				duration: 0,
				instrumental: false,
				plainLyrics: null,
				syncedLyrics: null
			} satisfies Lyrics);
		}

		const cached = locals.lrclibCacheRepo.getByLibraryItem(itemId);
		if (cached) {
			return json(rowToLyrics(cached));
		}
	}

	// Resolve MusicBrainz link
	const link = locals.libraryItemLinkRepo.getByItemAndService(itemId, 'musicbrainz');
	if (!link) {
		return json({ error: 'No MusicBrainz link for this item' }, { status: 404 });
	}

	// Get recording metadata (from MB cache or API)
	const mbid = link.service_id;
	let trackName: string;
	let artistName: string;
	let albumName: string | null = null;
	let durationSecs: number | null = null;

	const mbCached = locals.musicbrainzCacheRepo.getRecording(mbid);
	if (mbCached) {
		const display = JSON.parse(mbCached.data);
		trackName = display.title;
		artistName = display.artistCredits;
		albumName = display.firstReleaseTitle ?? null;
		durationSecs = display.durationMs ? Math.round(display.durationMs / 1000) : null;
	} else {
		const recording = await fetchRecording(mbid);
		if (!recording) {
			return json({ error: 'MusicBrainz recording not found' }, { status: 404 });
		}
		const display = recordingToDisplay(recording);
		locals.musicbrainzCacheRepo.upsertRecording(mbid, JSON.stringify(display));
		trackName = display.title;
		artistName = display.artistCredits;
		albumName = display.firstReleaseTitle ?? null;
		durationSecs = display.durationMs ? Math.round(display.durationMs / 1000) : null;
	}

	// Fetch lyrics from LRCLIB
	try {
		const result = await fetchLrcLibLyrics(trackName, artistName, albumName, durationSecs);

		if (result.status === 'not_found') {
			locals.lrclibCacheRepo.upsertLookup(itemId, null, 'not_found');
			return json({
				id: 0,
				trackName,
				artistName,
				albumName: albumName ?? '',
				duration: durationSecs ?? 0,
				instrumental: false,
				plainLyrics: null,
				syncedLyrics: null
			} satisfies Lyrics);
		}

		const lyrics = result.lyrics;
		const raw = result.raw;
		locals.lrclibCacheRepo.upsertLyrics(
			raw.id,
			raw.trackName,
			raw.artistName,
			raw.albumName,
			raw.duration,
			raw.instrumental,
			raw.plainLyrics,
			raw.syncedLyrics
		);
		locals.lrclibCacheRepo.upsertLookup(itemId, lyrics.id, 'found');

		return json(lyrics);
	} catch (err) {
		const message = err instanceof Error ? err.message : String(err);
		return json({ error: message }, { status: 500 });
	}
};
