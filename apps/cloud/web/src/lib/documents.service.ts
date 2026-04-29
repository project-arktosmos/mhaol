import { writable, type Writable } from 'svelte/store';

export interface Document {
	id: string;
	name: string;
	author: string;
	description: string;
	created_at: string;
	updated_at: string;
}

export interface DocumentsState {
	loading: boolean;
	documents: Document[];
	error: string | null;
}

const initialState: DocumentsState = {
	loading: false,
	documents: [],
	error: null
};

async function parseError(res: Response): Promise<string> {
	try {
		const data = await res.json();
		if (data && typeof data.error === 'string') return data.error;
	} catch {
		// fall through
	}
	return `HTTP ${res.status}`;
}

class DocumentsService {
	state: Writable<DocumentsState> = writable(initialState);

	async refresh(): Promise<void> {
		this.state.update((s) => ({ ...s, loading: true, error: null }));
		try {
			const res = await fetch('/api/documents', { cache: 'no-store' });
			if (!res.ok) throw new Error(await parseError(res));
			const documents = (await res.json()) as Document[];
			this.state.set({ loading: false, documents, error: null });
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Unknown error';
			this.state.update((s) => ({ ...s, loading: false, error: message }));
		}
	}

	async create(name: string, author: string, description: string): Promise<Document> {
		const res = await fetch('/api/documents', {
			method: 'POST',
			headers: { 'content-type': 'application/json' },
			body: JSON.stringify({ name, author, description })
		});
		if (!res.ok) throw new Error(await parseError(res));
		const created = (await res.json()) as Document;
		this.state.update((s) => ({ ...s, documents: [...s.documents, created] }));
		return created;
	}

	async remove(id: string): Promise<void> {
		const res = await fetch(`/api/documents/${encodeURIComponent(id)}`, { method: 'DELETE' });
		if (!res.ok && res.status !== 204) throw new Error(await parseError(res));
		this.state.update((s) => ({
			...s,
			documents: s.documents.filter((d) => d.id !== id)
		}));
	}
}

export const documentsService = new DocumentsService();
