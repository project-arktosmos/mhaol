import { writable, get, type Writable } from 'svelte/store';
import { fetchJson } from '$transport/fetch-helpers';
import type { SubsLyricsItem, SubsLyricsSearchType } from '$types/subs-lyrics.type';

interface SubsLyricsFinderState {
	type: SubsLyricsSearchType;
	query: string;
	externalId: string;
	searching: boolean;
	results: SubsLyricsItem[];
	error: string | null;
	selected: SubsLyricsItem | null;
}

const initialState: SubsLyricsFinderState = {
	type: 'album',
	query: '',
	externalId: '',
	searching: false,
	results: [],
	error: null,
	selected: null
};

class SubsLyricsFinderService {
	state: Writable<SubsLyricsFinderState> = writable(initialState);

	setType(type: SubsLyricsSearchType): void {
		this.state.update((s) => ({ ...s, type, results: [], error: null, selected: null }));
	}

	setQuery(query: string): void {
		this.state.update((s) => ({ ...s, query }));
	}

	setExternalId(externalId: string): void {
		this.state.update((s) => ({ ...s, externalId }));
	}

	clear(): void {
		this.state.set(initialState);
	}

	select(item: SubsLyricsItem | null): void {
		this.state.update((s) => ({ ...s, selected: item }));
	}

	async search(): Promise<void> {
		const current = get(this.state);
		const query = current.query.trim();
		const externalIds = current.externalId.trim() ? [current.externalId.trim()] : [];
		const isMusic = current.type === 'album';

		if (isMusic && !query) {
			this.state.update((s) => ({ ...s, error: 'Enter a query to search lyrics', results: [] }));
			return;
		}
		if (!isMusic && externalIds.length === 0) {
			this.state.update((s) => ({
				...s,
				error: 'Subtitle search needs a TMDB id',
				results: []
			}));
			return;
		}

		this.state.update((s) => ({ ...s, searching: true, error: null, selected: null }));

		try {
			const addon = addonForType(current.type);
			const results = await fetchJson<SubsLyricsItem[]>('/api/search/subs-lyrics', {
				method: 'POST',
				body: JSON.stringify({ addon, query, externalIds })
			});
			this.state.update((s) => ({ ...s, searching: false, results }));
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Unknown error';
			this.state.update((s) => ({ ...s, searching: false, error: message, results: [] }));
		}
	}
}

export const subsLyricsFinderService = new SubsLyricsFinderService();

function addonForType(type: SubsLyricsSearchType): string {
	switch (type) {
		case 'album':
			return 'lrclib';
		case 'movie':
			return 'wyzie-subs-movie';
		case 'tv show':
			return 'wyzie-subs-tv';
	}
}
