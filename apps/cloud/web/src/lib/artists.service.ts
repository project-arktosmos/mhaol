import { writable, type Writable } from 'svelte/store';

export interface Artist {
	id: string;
	name: string;
	role?: string;
	imageUrl?: string;
	created_at: string;
	updated_at: string;
}

export interface ArtistsState {
	loading: boolean;
	artists: Artist[];
	error: string | null;
}

const initialState: ArtistsState = {
	loading: false,
	artists: [],
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

class ArtistsService {
	state: Writable<ArtistsState> = writable(initialState);

	async refresh(): Promise<void> {
		this.state.update((s) => ({ ...s, loading: true, error: null }));
		try {
			const res = await fetch('/api/artists', { cache: 'no-store' });
			if (!res.ok) throw new Error(await parseError(res));
			const artists = (await res.json()) as Artist[];
			this.state.set({ loading: false, artists, error: null });
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Unknown error';
			this.state.update((s) => ({ ...s, loading: false, error: message }));
		}
	}

	async getOne(id: string): Promise<Artist> {
		const res = await fetch(`/api/artists/${encodeURIComponent(id)}`, { cache: 'no-store' });
		if (!res.ok) throw new Error(await parseError(res));
		return (await res.json()) as Artist;
	}

	async update(
		id: string,
		patch: { name: string; role?: string; imageUrl?: string }
	): Promise<Artist> {
		const res = await fetch(`/api/artists/${encodeURIComponent(id)}`, {
			method: 'PUT',
			headers: { 'content-type': 'application/json' },
			body: JSON.stringify(patch)
		});
		if (!res.ok) throw new Error(await parseError(res));
		const updated = (await res.json()) as Artist;
		this.state.update((s) => ({
			...s,
			artists: s.artists.map((a) => (a.id === id ? updated : a))
		}));
		return updated;
	}

	async remove(id: string): Promise<void> {
		const res = await fetch(`/api/artists/${encodeURIComponent(id)}`, { method: 'DELETE' });
		if (!res.ok && res.status !== 204) throw new Error(await parseError(res));
		this.state.update((s) => ({
			...s,
			artists: s.artists.filter((a) => a.id !== id)
		}));
	}
}

export const artistsService = new ArtistsService();
