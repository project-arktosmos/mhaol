import { describe, it, expect, vi, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import type { CloudDocument, DocumentFile } from '../../src/types/document.type';

// `vi.mock` is hoisted to the top of the file, so anything it captures must
// be created via `vi.hoisted` to exist at hoist time. Both the spy and the
// callback slot must be set up here so that the service module's
// constructor (which runs at import time, before the test body) sees the
// real spy and registers its TrackEnded callback into `registeredCallback`.
const mocks = vi.hoisted(() => ({
	playMock: vi.fn(),
	registeredCallback: null as (() => void) | null
}));
const playMock = mocks.playMock;

vi.mock('../../src/services/document-stream.service', async (importOriginal) => {
	const actual = await importOriginal<typeof import('../../src/services/document-stream.service')>();
	return {
		...actual,
		documentStreamService: { play: mocks.playMock }
	};
});

vi.mock('../../src/services/player.service', () => ({
	playerService: {
		onTrackEnded: (cb: () => void) => {
			mocks.registeredCallback = cb;
			return () => {
				if (mocks.registeredCallback === cb) mocks.registeredCallback = null;
			};
		}
	}
}));

const fireTrackEnded = () => mocks.registeredCallback?.();

import { documentPlaybackService } from '../../src/services/document-playback.service';

function track(title: string, value = `cid-${title}`): DocumentFile {
	return { type: 'ipfs', value, title };
}

function album(files: DocumentFile[]): CloudDocument {
	return {
		id: 'doc-album',
		title: 'Album Under Test',
		artists: [],
		description: '',
		images: [],
		files,
		year: 2024,
		type: 'album',
		source: 'musicbrainz',
		created_at: '2024-01-01',
		updated_at: '2024-01-01'
	};
}

describe('documentPlaybackService auto-advance', () => {
	beforeEach(() => {
		playMock.mockReset();
		documentPlaybackService.clear();
	});

	it('plays the first audio track when an album is selected', () => {
		const doc = album([track('01.mp3'), track('02.mp3'), track('03.mp3')]);

		documentPlaybackService.select(doc);

		expect(playMock).toHaveBeenCalledTimes(1);
		expect(playMock.mock.calls[0][0]).toMatchObject({ value: 'cid-01.mp3' });
		expect(get(documentPlaybackService.state).currentFile).toBe('cid-01.mp3');
	});

	it('advances to the next track when the worker reports TrackEnded', () => {
		const doc = album([track('01.mp3'), track('02.mp3'), track('03.mp3')]);
		documentPlaybackService.select(doc);
		expect(playMock).toHaveBeenCalledTimes(1);

		// Fire the TrackEnded callback the playback service registered with
		// playerService — this is what the data-channel handler will do
		// when the GStreamer pipeline emits EOS.
		expect(mocks.registeredCallback).not.toBeNull();
		fireTrackEnded();

		expect(playMock).toHaveBeenCalledTimes(2);
		expect(playMock.mock.calls[1][0]).toMatchObject({ value: 'cid-02.mp3' });
		expect(get(documentPlaybackService.state).currentFile).toBe('cid-02.mp3');
	});

	it('stops advancing past the last track', () => {
		const doc = album([track('01.mp3'), track('02.mp3')]);
		documentPlaybackService.select(doc);
		fireTrackEnded(); // → 02
		fireTrackEnded(); // no-op (last track)

		expect(playMock).toHaveBeenCalledTimes(2);
		// currentFile sticks at the last played track (02.mp3), not cleared.
		expect(get(documentPlaybackService.state).currentFile).toBe('cid-02.mp3');
	});

	it('skips non-playable files when advancing (e.g. cover.jpg between tracks)', () => {
		const doc = album([
			track('01.mp3'),
			{ type: 'ipfs', value: 'cid-cover.jpg', title: 'cover.jpg' },
			track('02.mp3')
		]);
		documentPlaybackService.select(doc);
		expect(playMock.mock.calls[0][0]).toMatchObject({ value: 'cid-01.mp3' });

		fireTrackEnded();

		// Cover image is filtered out of the playable list, so the next file
		// is 02.mp3 — not the cover.
		expect(playMock).toHaveBeenCalledTimes(2);
		expect(playMock.mock.calls[1][0]).toMatchObject({ value: 'cid-02.mp3' });
	});

	it('does not advance for non-album documents (e.g. movies)', () => {
		const movie: CloudDocument = {
			...album([track('movie.mp4', 'cid-movie')]),
			type: 'movie'
		};
		movie.files = [{ type: 'ipfs', value: 'cid-movie', title: 'movie.mp4' }];

		documentPlaybackService.select(movie);

		// Single video file → autoplays.
		expect(playMock).toHaveBeenCalledTimes(1);

		fireTrackEnded();
		// Only one playable file in the list; advance is a no-op.
		expect(playMock).toHaveBeenCalledTimes(1);
	});

	it('does not autoplay when album has no audio files', () => {
		const doc: CloudDocument = album([
			{ type: 'ipfs', value: 'cid-cover.jpg', title: 'cover.jpg' }
		]);

		documentPlaybackService.select(doc);

		expect(playMock).not.toHaveBeenCalled();
		expect(get(documentPlaybackService.state).currentFile).toBeNull();
	});
});
