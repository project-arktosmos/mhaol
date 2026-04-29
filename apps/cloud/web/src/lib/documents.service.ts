import { writable, type Writable } from 'svelte/store';

export const DOCUMENT_TYPES = [
	'movie',
	'tv season',
	'tv episode',
	'tv show',
	'album',
	'track',
	'image',
	'youtube video',
	'youtube channel',
	'book',
	'game'
] as const;

export type DocumentType = (typeof DOCUMENT_TYPES)[number];

export const DOCUMENT_SOURCES = [
	'tmdb',
	'musicbrainz',
	'retroachievements',
	'youtube',
	'lrclib',
	'openlibrary',
	'wyzie-subs'
] as const;

export type DocumentSource = (typeof DOCUMENT_SOURCES)[number];

export const TYPES_BY_SOURCE: Record<DocumentSource, readonly DocumentType[]> = {
	tmdb: ['movie', 'tv show', 'tv season', 'tv episode', 'image'],
	musicbrainz: ['album', 'track'],
	retroachievements: ['game'],
	youtube: ['youtube video', 'youtube channel'],
	lrclib: ['track'],
	openlibrary: ['book'],
	'wyzie-subs': ['movie', 'tv episode']
};

export interface Artist {
	name: string;
	url?: string;
	imageUrl?: string;
}

export interface ImageMeta {
	url: string;
	mimeType: string;
	fileSize: number;
	width: number;
	height: number;
}

export const FILE_TYPES = ['ipfs', 'torrent magnet', 'url'] as const;
export type FileType = (typeof FILE_TYPES)[number];

export interface FileEntry {
	type: FileType;
	value: string;
	title?: string;
}

export interface Document {
	id: string;
	title: string;
	artists: Artist[];
	description: string;
	images: ImageMeta[];
	files: FileEntry[];
	year: number | null;
	type: string;
	source: string;
	created_at: string;
	updated_at: string;
	version?: number;
	version_hashes?: string[];
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

const POLL_INTERVAL_MS = 4000;

class DocumentsService {
	state: Writable<DocumentsState> = writable(initialState);

	private subscribers = 0;
	private timer: ReturnType<typeof setInterval> | null = null;
	private inFlight = false;

	start(): () => void {
		this.subscribers += 1;
		if (this.subscribers === 1) {
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
		if (this.inFlight) return;
		this.inFlight = true;
		this.state.update((s) => ({ ...s, loading: true, error: null }));
		try {
			const res = await fetch('/api/documents', { cache: 'no-store' });
			if (!res.ok) throw new Error(await parseError(res));
			const documents = (await res.json()) as Document[];
			this.state.set({ loading: false, documents, error: null });
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Unknown error';
			this.state.update((s) => ({ ...s, loading: false, error: message }));
		} finally {
			this.inFlight = false;
		}
	}

	async create(input: {
		title: string;
		artists: Artist[];
		description: string;
		images: ImageMeta[];
		files: FileEntry[];
		year: number | null;
		type: DocumentType;
		source: DocumentSource;
	}): Promise<Document> {
		const res = await fetch('/api/documents', {
			method: 'POST',
			headers: { 'content-type': 'application/json' },
			body: JSON.stringify(input)
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
