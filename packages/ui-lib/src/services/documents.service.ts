import { writable, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { fetchJson } from 'ui-lib/transport/fetch-helpers';
import type { CloudDocument } from 'ui-lib/types/document.type';

export interface DocumentsServiceState {
	loading: boolean;
	documents: CloudDocument[];
	error: string | null;
}

const initialState: DocumentsServiceState = {
	loading: false,
	documents: [],
	error: null
};

class DocumentsService {
	public state: Writable<DocumentsServiceState> = writable(initialState);

	async refresh(): Promise<void> {
		if (!browser) return;
		this.state.update((s) => ({ ...s, loading: true, error: null }));
		try {
			const documents = await fetchJson<CloudDocument[]>('/api/documents');
			this.state.set({ loading: false, documents, error: null });
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Unknown error';
			this.state.update((s) => ({ ...s, loading: false, error: message }));
		}
	}
}

export const documentsService = new DocumentsService();
