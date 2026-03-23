import { writable } from 'svelte/store';
import { apiUrl } from 'ui-lib/lib/api-base';
import type { SmartPairItem, SmartPairResult, SmartPairState } from 'ui-lib/types/smart-pair.type';
import type { DisplayTMDBMovie, DisplayTMDBTvShow } from 'addons/tmdb/types';

const initialState: SmartPairState = {
	pairing: false,
	results: [],
	error: null
};

class SmartPairService {
	store = writable<SmartPairState>(initialState);

	async pair(items: SmartPairItem[]) {
		this.store.set({ ...initialState, pairing: true });

		try {
			const res = await fetch(apiUrl('/api/smart-pair/pair'), {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ items })
			});

			if (!res.ok) {
				const text = await res.text().catch(() => '');
				this.store.update((s) => ({
					...s,
					pairing: false,
					error: `Pairing failed: ${res.status} ${text || res.statusText}`
				}));
				return;
			}

			const reader = res.body?.getReader();
			if (!reader) {
				this.store.update((s) => ({ ...s, pairing: false, error: 'No response stream' }));
				return;
			}

			const decoder = new TextDecoder();
			let buffer = '';

			while (true) {
				const { done, value } = await reader.read();
				if (done) break;

				buffer += decoder.decode(value, { stream: true });
				const lines = buffer.split('\n');
				buffer = lines.pop() ?? '';

				for (const line of lines) {
					if (!line.trim()) continue;
					try {
						const result: SmartPairResult = JSON.parse(line);
						this.store.update((s) => ({
							...s,
							results: [...s.results, result]
						}));
					} catch {
						// skip malformed lines
					}
				}
			}

			this.store.update((s) => ({ ...s, pairing: false }));
		} catch (e) {
			this.store.update((s) => ({
				...s,
				pairing: false,
				error: `Pairing failed: ${e instanceof Error ? e.message : String(e)}`
			}));
		}
	}

	async loadPinned(): Promise<{ movies: DisplayTMDBMovie[]; tv: DisplayTMDBTvShow[] }> {
		try {
			const res = await fetch(apiUrl('/api/smart-pair/pinned'));
			if (res.ok) {
				return await res.json();
			}
		} catch {
			// best-effort
		}
		return { movies: [], tv: [] };
	}

	reset() {
		this.store.set(initialState);
	}
}

export const smartPairService = new SmartPairService();
