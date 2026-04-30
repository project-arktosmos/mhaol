import { writable, type Writable } from 'svelte/store';

export const FIRKIN_TYPES = [
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
	'game',
	'iptv channel',
	'radio station'
] as const;

export type FirkinType = (typeof FIRKIN_TYPES)[number];

export const FIRKIN_SOURCES = [
	'tmdb',
	'musicbrainz',
	'retroachievements',
	'youtube',
	'lrclib',
	'openlibrary',
	'wyzie-subs',
	'iptv',
	'radio'
] as const;

export type FirkinSource = (typeof FIRKIN_SOURCES)[number];

export const TYPES_BY_SOURCE: Record<FirkinSource, readonly FirkinType[]> = {
	tmdb: ['movie', 'tv show', 'tv season', 'tv episode', 'image'],
	musicbrainz: ['album', 'track'],
	retroachievements: ['game'],
	youtube: ['youtube video', 'youtube channel'],
	lrclib: ['track'],
	openlibrary: ['book'],
	'wyzie-subs': ['movie', 'tv episode'],
	iptv: ['iptv channel'],
	radio: ['radio station']
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

export interface Firkin {
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

export interface FirkinsState {
	loading: boolean;
	firkins: Firkin[];
	error: string | null;
}

const initialState: FirkinsState = {
	loading: false,
	firkins: [],
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

class FirkinsService {
	state: Writable<FirkinsState> = writable(initialState);

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
			const res = await fetch('/api/firkins', { cache: 'no-store' });
			if (!res.ok) throw new Error(await parseError(res));
			const firkins = (await res.json()) as Firkin[];
			this.state.set({ loading: false, firkins, error: null });
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
		type: FirkinType;
		source: FirkinSource;
	}): Promise<Firkin> {
		const res = await fetch('/api/firkins', {
			method: 'POST',
			headers: { 'content-type': 'application/json' },
			body: JSON.stringify(input)
		});
		if (!res.ok) throw new Error(await parseError(res));
		const created = (await res.json()) as Firkin;
		this.state.update((s) => {
			const existing = s.firkins.findIndex((d) => d.id === created.id);
			if (existing >= 0) {
				const next = s.firkins.slice();
				next[existing] = created;
				return { ...s, firkins: next };
			}
			return { ...s, firkins: [...s.firkins, created] };
		});
		return created;
	}

	async remove(id: string): Promise<void> {
		const res = await fetch(`/api/firkins/${encodeURIComponent(id)}`, { method: 'DELETE' });
		if (!res.ok && res.status !== 204) throw new Error(await parseError(res));
		this.state.update((s) => ({
			...s,
			firkins: s.firkins.filter((d) => d.id !== id)
		}));
	}
}

export const firkinsService = new FirkinsService();
