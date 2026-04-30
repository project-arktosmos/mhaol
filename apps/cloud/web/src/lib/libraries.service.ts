import { writable, type Writable } from 'svelte/store';
import type { IpfsPin } from '$lib/ipfs.service';

export const LIBRARY_KINDS = ['movie', 'tv', 'album', 'book', 'game'] as const;
export type LibraryKind = (typeof LIBRARY_KINDS)[number];

export const LIBRARY_KIND_LABELS: Record<LibraryKind, string> = {
	movie: 'Movies',
	tv: 'TV Shows',
	album: 'Albums',
	book: 'Books',
	game: 'Games'
};

export interface Library {
	id: string;
	path: string;
	kinds: LibraryKind[];
	created_at: string;
	updated_at: string;
	last_scanned_at: string | null;
}

export interface ScanEntry {
	path: string;
	relative_path: string;
	size: number;
	mime: string;
}

export interface ScanResponse {
	root: string;
	total_files: number;
	total_size: number;
	entries: ScanEntry[];
}

export interface LibrariesState {
	loading: boolean;
	libraries: Library[];
	error: string | null;
}

const initialState: LibrariesState = {
	loading: false,
	libraries: [],
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

class LibrariesService {
	state: Writable<LibrariesState> = writable(initialState);

	async refresh(): Promise<void> {
		this.state.update((s) => ({ ...s, loading: true, error: null }));
		try {
			const res = await fetch('/api/libraries', { cache: 'no-store' });
			if (!res.ok) throw new Error(await parseError(res));
			const libraries = (await res.json()) as Library[];
			this.state.set({ loading: false, libraries, error: null });
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Unknown error';
			this.state.update((s) => ({ ...s, loading: false, error: message }));
		}
	}

	async create(path: string, kinds: LibraryKind[] = []): Promise<Library> {
		const res = await fetch('/api/libraries', {
			method: 'POST',
			headers: { 'content-type': 'application/json' },
			body: JSON.stringify({ path, kinds })
		});
		if (!res.ok) throw new Error(await parseError(res));
		const created = (await res.json()) as Library;
		this.state.update((s) => ({ ...s, libraries: [...s.libraries, created] }));
		return created;
	}

	async update(id: string, patch: { path?: string; kinds?: LibraryKind[] }): Promise<Library> {
		const current = this.findInState(id);
		const body = {
			path: patch.path ?? current?.path ?? '',
			...(patch.kinds === undefined ? {} : { kinds: patch.kinds })
		};
		const res = await fetch(`/api/libraries/${encodeURIComponent(id)}`, {
			method: 'PUT',
			headers: { 'content-type': 'application/json' },
			body: JSON.stringify(body)
		});
		if (!res.ok) throw new Error(await parseError(res));
		const updated = (await res.json()) as Library;
		this.state.update((s) => ({
			...s,
			libraries: s.libraries.map((l) => (l.id === id ? updated : l))
		}));
		return updated;
	}

	private findInState(id: string): Library | undefined {
		let found: Library | undefined;
		this.state.subscribe((s) => {
			found = s.libraries.find((l) => l.id === id);
		})();
		return found;
	}

	async scan(id: string): Promise<ScanResponse> {
		const res = await fetch(`/api/libraries/${encodeURIComponent(id)}/scan`, {
			cache: 'no-store'
		});
		if (!res.ok) throw new Error(await parseError(res));
		return (await res.json()) as ScanResponse;
	}

	async pins(id: string): Promise<IpfsPin[]> {
		const res = await fetch(`/api/libraries/${encodeURIComponent(id)}/pins`, {
			cache: 'no-store'
		});
		if (!res.ok) throw new Error(await parseError(res));
		return (await res.json()) as IpfsPin[];
	}

	async remove(id: string): Promise<void> {
		const res = await fetch(`/api/libraries/${encodeURIComponent(id)}`, { method: 'DELETE' });
		if (!res.ok && res.status !== 204) throw new Error(await parseError(res));
		this.state.update((s) => ({
			...s,
			libraries: s.libraries.filter((l) => l.id !== id)
		}));
	}
}

export const librariesService = new LibrariesService();
