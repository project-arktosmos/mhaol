import { writable, type Writable } from 'svelte/store';
import type { CloudDocument, DocumentFile } from 'ui-lib/types/document.type';

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
	}

	clear(): void {
		this.state.set(initialState);
	}
}

export const documentPlaybackService = new DocumentPlaybackService();
