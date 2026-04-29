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

const POLL_INTERVAL_MS = 4000;

class DocumentsService {
	public state: Writable<DocumentsServiceState> = writable(initialState);

	private subscribers = 0;
	private timer: ReturnType<typeof setInterval> | null = null;
	private inFlight = false;

	/** Begin refcounted polling. Returns a cleanup that stops polling once the last consumer leaves. */
	start(): () => void {
		this.subscribers += 1;
		if (this.subscribers === 1 && browser) {
			void this.refresh();
			this.timer = setInterval(() => {
				void this.refresh();
			}, POLL_INTERVAL_MS);
		}
		return () => this.stop();
	}

	private stop(): void {
		this.subscribers = Math.max(0, this.subscribers - 1);
		if (this.subscribers === 0 && this.timer) {
			clearInterval(this.timer);
			this.timer = null;
		}
	}

	async refresh(): Promise<void> {
		if (!browser || this.inFlight) return;
		this.inFlight = true;
		this.state.update((s) => ({ ...s, loading: true, error: null }));
		try {
			const documents = await fetchJson<CloudDocument[]>('/api/documents');
			this.state.set({ loading: false, documents, error: null });
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Unknown error';
			this.state.update((s) => ({ ...s, loading: false, error: message }));
		} finally {
			this.inFlight = false;
		}
	}
}

export const documentsService = new DocumentsService();
