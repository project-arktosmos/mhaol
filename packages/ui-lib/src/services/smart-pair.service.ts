import { writable } from 'svelte/store';
import { apiUrl } from 'ui-lib/lib/api-base';
import type { SmartPairItem, SmartPairResult, SmartPairState } from 'ui-lib/types/smart-pair.type';
import type { DisplayTMDBMovie, DisplayTMDBTvShow } from 'addons/tmdb/types';

const initialState: SmartPairState = {
	pairing: false,
	results: [],
	saving: false,
	saved: false,
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

			const data: { results: Omit<SmartPairResult, 'accepted'>[] } = await res.json();
			const results: SmartPairResult[] = data.results.map((r) => ({
				...r,
				accepted: r.confidence === 'high' || r.confidence === 'medium'
			}));

			this.store.update((s) => ({ ...s, pairing: false, results }));
		} catch (e) {
			this.store.update((s) => ({
				...s,
				pairing: false,
				error: `Pairing failed: ${e instanceof Error ? e.message : String(e)}`
			}));
		}
	}

	toggleResult(sourceId: string) {
		this.store.update((s) => ({
			...s,
			results: s.results.map((r) => (r.sourceId === sourceId ? { ...r, accepted: !r.accepted } : r))
		}));
	}

	acceptAll() {
		this.store.update((s) => ({
			...s,
			results: s.results.map((r) => ({ ...r, accepted: r.matched }))
		}));
	}

	rejectAll() {
		this.store.update((s) => ({
			...s,
			results: s.results.map((r) => ({ ...r, accepted: false }))
		}));
	}

	async save() {
		let state: SmartPairState = initialState;
		this.store.subscribe((s) => (state = s))();

		const accepted = state.results.filter((r) => r.accepted && r.matched && r.tmdbId);
		if (accepted.length === 0) return;

		this.store.update((s) => ({ ...s, saving: true, error: null }));

		try {
			const res = await fetch(apiUrl('/api/smart-pair/save'), {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					items: accepted.map((r) => ({
						sourceId: r.sourceId,
						source: r.source,
						title: r.sourceTitle,
						tmdbId: r.tmdbId,
						tmdbType: r.tmdbType
					}))
				})
			});

			if (!res.ok) {
				const text = await res.text().catch(() => '');
				this.store.update((s) => ({
					...s,
					saving: false,
					error: `Save failed: ${res.status} ${text || res.statusText}`
				}));
				return;
			}

			this.store.update((s) => ({ ...s, saving: false, saved: true }));
		} catch (e) {
			this.store.update((s) => ({
				...s,
				saving: false,
				error: `Save failed: ${e instanceof Error ? e.message : String(e)}`
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
