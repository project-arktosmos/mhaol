import { writable, type Writable } from 'svelte/store';
import type { CloudFirkin, FirkinFile } from 'ui-lib/types/firkin.type';
import {
	firkinStreamService,
	isAudioFile,
	isPlayableFile,
	isVideoFile
} from 'ui-lib/services/firkin-stream.service';
import { playerService } from 'ui-lib/services/player.service';

/// Addons whose firkins represent an album-shaped collection of audio
/// files. Used to decide whether a firkin should auto-play the first audio
/// track when no video file is present.
const ALBUM_ADDONS = new Set<string>(['musicbrainz', 'lrclib', 'local-album']);

function isAlbumAddon(addon: string): boolean {
	return ALBUM_ADDONS.has(addon);
}

export interface FirkinPlaybackState {
	firkin: CloudFirkin | null;
	files: FirkinFile[];
	/// Currently-playing file's value (CID for IPFS files), or null when
	/// nothing is playing or the user has stopped playback.
	currentFile: string | null;
}

const initialState: FirkinPlaybackState = {
	firkin: null,
	files: [],
	currentFile: null
};

class FirkinPlaybackService {
	state: Writable<FirkinPlaybackState> = writable(initialState);
	private unsubscribeTrackEnded: (() => void) | null = null;

	constructor() {
		this.unsubscribeTrackEnded = playerService.onTrackEnded(() => this.advance());
	}

	select(firkin: CloudFirkin): void {
		const files = (firkin.files ?? []).filter((f) => f.type === 'ipfs');
		this.state.set({ firkin, files, currentFile: null });

		const videos = files.filter(isVideoFile);
		if (videos.length === 1) {
			this.play(videos[0]);
			return;
		}

		const audios = files.filter(isAudioFile);
		if (isAlbumAddon(firkin.addon) && audios.length > 0) {
			this.play(audios[0]);
		}
	}

	/// Play a specific file from the current firkin and remember which one
	/// it is so `advance()` can find the next playable file.
	play(file: FirkinFile): void {
		this.state.update((s) => ({ ...s, currentFile: file.value }));
		void firkinStreamService.play(file);
	}

	/// Advance to the next playable file after `currentFile` in the firkin.
	/// No-op if there is no next playable file. Called automatically when the
	/// worker signals `TrackEnded`.
	advance(): void {
		let next: FirkinFile | null = null;
		this.state.update((s) => {
			const playable = s.files.filter(isPlayableFile);
			if (s.currentFile == null || playable.length === 0) return s;
			const idx = playable.findIndex((f) => f.value === s.currentFile);
			if (idx === -1 || idx >= playable.length - 1) return s;
			next = playable[idx + 1];
			return { ...s, currentFile: next.value };
		});
		if (next) {
			void firkinStreamService.play(next);
		}
	}

	clear(): void {
		this.state.set(initialState);
	}
}

export const firkinPlaybackService = new FirkinPlaybackService();
