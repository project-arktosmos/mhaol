import { writable, type Readable } from 'svelte/store';
import { fetchRaw } from '$transport/fetch-helpers';
import { getPosterUrl, getBackdropUrl } from 'addons/tmdb/transform';
import type { DisplayTMDBMovie } from 'addons/tmdb/types';

interface ImageOverrideEntry {
	tmdb_id: number;
	role: string;
	file_path: string;
}

class ImageOverridesService {
	private _state = writable<Map<number, Record<string, string>>>(new Map());

	get state(): Readable<Map<number, Record<string, string>>> {
		return this._state;
	}

	async load(mediaType: 'movie' | 'tv'): Promise<void> {
		try {
			const res = await fetchRaw(`/api/tmdb/image-overrides/${mediaType}`);
			if (!res.ok) return;
			const entries: ImageOverrideEntry[] = await res.json();
			const map = new Map<number, Record<string, string>>();
			for (const o of entries) {
				const existing = map.get(o.tmdb_id) ?? {};
				existing[o.role] = o.file_path;
				map.set(o.tmdb_id, existing);
			}
			this._state.set(map);
		} catch {
			/* best-effort */
		}
	}

	applyToMovies(
		movies: DisplayTMDBMovie[],
		overrides: Map<number, Record<string, string>>
	): DisplayTMDBMovie[] {
		if (overrides.size === 0) return movies;
		return movies.map((m) => {
			const o = overrides.get(m.id);
			if (!o) return m;
			return {
				...m,
				posterUrl: o.poster ? getPosterUrl(o.poster) : m.posterUrl,
				backdropUrl: o.backdrop ? getBackdropUrl(o.backdrop) : m.backdropUrl
			};
		});
	}
}

export const imageOverridesService = new ImageOverridesService();
