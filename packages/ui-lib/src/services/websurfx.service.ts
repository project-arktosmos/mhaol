import { writable, type Writable } from 'svelte/store';
import { fetchJson } from 'ui-lib/transport/fetch-helpers';
import type { WebSurfxSearchResponse, WebSurfxState } from 'ui-lib/types/websurfx.type';

const initialState: WebSurfxState = {
	query: '',
	searching: false,
	results: [],
	provider: null,
	error: null
};

class WebSurfxService {
	public state: Writable<WebSurfxState> = writable(initialState);

	async search(query: string): Promise<void> {
		if (!query.trim()) return;

		this.state.update((s) => ({
			...s,
			query,
			searching: true,
			error: null
		}));

		try {
			const data = await fetchJson<WebSurfxSearchResponse>(
				`/api/websurfx/search?q=${encodeURIComponent(query)}`
			);
			this.state.update((s) => ({
				...s,
				searching: false,
				results: data.results,
				provider: data.provider
			}));
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Search failed';
			this.state.update((s) => ({
				...s,
				searching: false,
				error: message
			}));
		}
	}

	clearResults(): void {
		this.state.set(initialState);
	}
}

export const websurfxService = new WebSurfxService();
