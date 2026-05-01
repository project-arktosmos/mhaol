import { writable, type Writable } from 'svelte/store';
import type { IpfsPin } from '$lib/ipfs.service';
import type { Firkin } from '$lib/firkins.service';

/// The local-* addon ids a library may declare it contains. Each addon
/// represents a single firkin kind (movie, tv show, album, book, game), so
/// the library no longer stores a separate `kinds` enum — it stores the
/// addon ids directly.
export const LIBRARY_ADDONS = [
	'local-movie',
	'local-tv',
	'local-album',
	'local-book',
	'local-game'
] as const;
export type LibraryAddon = (typeof LIBRARY_ADDONS)[number];

export const LIBRARY_ADDON_LABELS: Record<LibraryAddon, string> = {
	'local-movie': 'Movies',
	'local-tv': 'TV Shows',
	'local-album': 'Albums',
	'local-book': 'Books',
	'local-game': 'Games'
};

export interface Library {
	id: string;
	path: string;
	addons: LibraryAddon[];
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

	async create(path: string, addons: LibraryAddon[] = []): Promise<Library> {
		const res = await fetch('/api/libraries', {
			method: 'POST',
			headers: { 'content-type': 'application/json' },
			body: JSON.stringify({ path, addons })
		});
		if (!res.ok) throw new Error(await parseError(res));
		const created = (await res.json()) as Library;
		this.state.update((s) => ({ ...s, libraries: [...s.libraries, created] }));
		return created;
	}

	async update(id: string, patch: { path?: string; addons?: LibraryAddon[] }): Promise<Library> {
		const current = this.findInState(id);
		const body = {
			path: patch.path ?? current?.path ?? '',
			...(patch.addons === undefined ? {} : { addons: patch.addons })
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

	async firkins(id: string): Promise<Firkin[]> {
		const res = await fetch(`/api/libraries/${encodeURIComponent(id)}/firkins`, {
			cache: 'no-store'
		});
		if (!res.ok) throw new Error(await parseError(res));
		return (await res.json()) as Firkin[];
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
