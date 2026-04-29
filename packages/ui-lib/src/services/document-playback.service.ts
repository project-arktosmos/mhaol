import { writable, type Writable } from 'svelte/store';
import type { CloudDocument, DocumentFile } from 'ui-lib/types/document.type';
import {
	documentStreamService,
	isAudioFile,
	isPlayableFile,
	isVideoFile
} from 'ui-lib/services/document-stream.service';
import { playerService } from 'ui-lib/services/player.service';

export interface DocumentPlaybackState {
	document: CloudDocument | null;
	files: DocumentFile[];
	/// Currently-playing file's value (CID for IPFS files), or null when
	/// nothing is playing or the user has stopped playback.
	currentFile: string | null;
}

const initialState: DocumentPlaybackState = {
	document: null,
	files: [],
	currentFile: null
};

class DocumentPlaybackService {
	state: Writable<DocumentPlaybackState> = writable(initialState);
	private unsubscribeTrackEnded: (() => void) | null = null;

	constructor() {
		this.unsubscribeTrackEnded = playerService.onTrackEnded(() => this.advance());
	}

	select(document: CloudDocument): void {
		const files = (document.files ?? []).filter((f) => f.type === 'ipfs');
		this.state.set({ document, files, currentFile: null });

		const videos = files.filter(isVideoFile);
		if (videos.length === 1) {
			this.play(videos[0]);
			return;
		}

		const audios = files.filter(isAudioFile);
		if (document.type === 'album' && audios.length > 0) {
			this.play(audios[0]);
		}
	}

	/// Play a specific file from the current document and remember which one
	/// it is so `advance()` can find the next playable file.
	play(file: DocumentFile): void {
		this.state.update((s) => ({ ...s, currentFile: file.value }));
		void documentStreamService.play(file);
	}

	/// Advance to the next playable file after `currentFile` in the document.
	/// No-op if there is no next playable file. Called automatically when the
	/// worker signals `TrackEnded`.
	advance(): void {
		let next: DocumentFile | null = null;
		this.state.update((s) => {
			const playable = s.files.filter(isPlayableFile);
			if (s.currentFile == null || playable.length === 0) return s;
			const idx = playable.findIndex((f) => f.value === s.currentFile);
			if (idx === -1 || idx >= playable.length - 1) return s;
			next = playable[idx + 1];
			return { ...s, currentFile: next.value };
		});
		if (next) {
			void documentStreamService.play(next);
		}
	}

	clear(): void {
		this.state.set(initialState);
	}
}

export const documentPlaybackService = new DocumentPlaybackService();
