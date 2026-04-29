import { writable, type Writable } from 'svelte/store';
import type { IpfsPin } from '$lib/ipfs.service';

export interface Library {
	id: string;
	path: string;
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

	async create(path: string): Promise<Library> {
		const res = await fetch('/api/libraries', {
			method: 'POST',
			headers: { 'content-type': 'application/json' },
			body: JSON.stringify({ path })
		});
		if (!res.ok) throw new Error(await parseError(res));
		const created = (await res.json()) as Library;
		this.state.update((s) => ({ ...s, libraries: [...s.libraries, created] }));
		return created;
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
