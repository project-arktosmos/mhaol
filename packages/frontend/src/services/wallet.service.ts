import { writable, get, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { apiUrl } from 'frontend/lib/api-base';

export interface WalletState {
	loading: boolean;
	name: string | null;
	address: string | null;
	error: string | null;
}

const initialState: WalletState = {
	loading: false,
	name: null,
	address: null,
	error: null
};

class WalletService {
	public state: Writable<WalletState> = writable(initialState);

	private _initialized = false;

	async initialize(): Promise<void> {
		if (!browser || this._initialized) return;

		this.state.update((s) => ({ ...s, loading: true, error: null }));

		try {
			const res = await fetch(apiUrl('/api/signaling/wallet'));
			if (!res.ok) throw new Error(`HTTP ${res.status}`);

			const data = (await res.json()) as { name: string; address: string };

			this._initialized = true;
			this.state.update((s) => ({
				...s,
				loading: false,
				name: data.name,
				address: data.address
			}));
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Failed to load wallet';
			this.state.update((s) => ({ ...s, loading: false, error: message }));
		}
	}

	async regenerate(): Promise<void> {
		this.state.update((s) => ({ ...s, loading: true, error: null }));

		try {
			const res = await fetch(apiUrl('/api/signaling/wallet'), { method: 'DELETE' });
			if (!res.ok) throw new Error(`HTTP ${res.status}`);

			const data = (await res.json()) as { name: string; address: string };

			this.state.update((s) => ({
				...s,
				loading: false,
				name: data.name,
				address: data.address
			}));
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Failed to regenerate wallet';
			this.state.update((s) => ({ ...s, loading: false, error: message }));
		}
	}

	async sign(message: string): Promise<string | null> {
		try {
			const res = await fetch(apiUrl('/api/signaling/wallet/sign'), {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ message })
			});
			if (!res.ok) return null;

			const data = (await res.json()) as { signature: string };
			return data.signature;
		} catch {
			return null;
		}
	}

	getAddress(): string | null {
		return get(this.state).address;
	}
}

export const walletService = new WalletService();
