import { writable, type Writable } from 'svelte/store';

export interface Library {
	id: string;
	name: string;
	path: string;
	created_at: string;
	updated_at: string;
}

export interface LibraryDefaults {
	base: string;
}

export interface LibrariesState {
	loading: boolean;
	libraries: Library[];
	defaults: LibraryDefaults | null;
	error: string | null;
}

const initialState: LibrariesState = {
	loading: false,
	libraries: [],
	defaults: null,
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
			const [listRes, defaultsRes] = await Promise.all([
				fetch('/api/libraries', { cache: 'no-store' }),
				fetch('/api/libraries/defaults', { cache: 'no-store' })
			]);
			if (!listRes.ok) throw new Error(await parseError(listRes));
			if (!defaultsRes.ok) throw new Error(await parseError(defaultsRes));
			const libraries = (await listRes.json()) as Library[];
			const defaults = (await defaultsRes.json()) as LibraryDefaults;
			this.state.set({
				loading: false,
				libraries,
				defaults,
				error: null
			});
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Unknown error';
			this.state.update((s) => ({ ...s, loading: false, error: message }));
		}
	}

	async create(name: string, path?: string): Promise<Library> {
		const body: Record<string, string> = { name };
		if (path && path.trim() !== '') body.path = path.trim();
		const res = await fetch('/api/libraries', {
			method: 'POST',
			headers: { 'content-type': 'application/json' },
			body: JSON.stringify(body)
		});
		if (!res.ok) throw new Error(await parseError(res));
		const created = (await res.json()) as Library;
		this.state.update((s) => ({ ...s, libraries: [...s.libraries, created] }));
		return created;
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
