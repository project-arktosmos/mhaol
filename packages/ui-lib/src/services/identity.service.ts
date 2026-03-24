import { writable, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { fetchRaw } from 'ui-lib/transport/fetch-helpers';
import type { Identity } from 'ui-lib/types/identity.type';

interface IdentityState {
	loading: boolean;
	identities: Identity[];
	error: string | null;
}

const initialState: IdentityState = {
	loading: false,
	identities: [],
	error: null
};

class IdentityService {
	public state: Writable<IdentityState> = writable(initialState);

	private _initialized = false;

	async initialize(): Promise<void> {
		if (!browser || this._initialized) return;
		this._initialized = true;
		await this.refresh();
	}

	async refresh(): Promise<void> {
		this.state.update((s) => ({ ...s, loading: true, error: null }));
		try {
			const res = await fetchRaw('/api/identities');
			if (!res.ok) throw new Error(`HTTP ${res.status}`);
			const identities: Identity[] = await res.json();
			this.state.update((s) => ({ ...s, loading: false, identities }));
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Failed to load identities';
			this.state.update((s) => ({ ...s, loading: false, error: message }));
		}
	}
}

export const identityService = new IdentityService();
