import { writable, type Writable } from 'svelte/store';
import type { CloudDocument, DocumentFile } from 'ui-lib/types/document.type';
import {
	documentStreamService,
	isAudioFile,
	isVideoFile
} from 'ui-lib/services/document-stream.service';

export interface DocumentPlaybackState {
	document: CloudDocument | null;
	files: DocumentFile[];
}

const initialState: DocumentPlaybackState = {
	document: null,
	files: []
};

class DocumentPlaybackService {
	state: Writable<DocumentPlaybackState> = writable(initialState);

	select(document: CloudDocument): void {
		const files = (document.files ?? []).filter((f) => f.type === 'ipfs');
		this.state.set({ document, files });

		const videos = files.filter(isVideoFile);
		if (videos.length === 1) {
			void documentStreamService.play(videos[0]);
			return;
		}

		const audios = files.filter(isAudioFile);
		if (document.type === 'album' && audios.length > 0) {
			void documentStreamService.play(audios[0]);
		}
	}

	clear(): void {
		this.state.set(initialState);
	}
}

export const documentPlaybackService = new DocumentPlaybackService();
