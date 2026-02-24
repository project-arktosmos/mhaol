import { writable, get, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { privateKeyToAccount } from 'viem/accounts';

export interface WalletState {
	loading: boolean;
	address: string | null;
	privateKey: string | null;
	error: string | null;
}

const initialState: WalletState = {
	loading: false,
	address: null,
	privateKey: null,
	error: null
};

class WalletService {
	public state: Writable<WalletState> = writable(initialState);

	private _initialized = false;

	async initialize(): Promise<void> {
		if (!browser || this._initialized) return;

		this.state.update((s) => ({ ...s, loading: true, error: null }));

		try {
			const res = await fetch('/api/signaling/wallet');
			if (!res.ok) throw new Error(`HTTP ${res.status}`);

			const data = (await res.json()) as { privateKey: string; address: string };

			this._initialized = true;
			this.state.update((s) => ({
				...s,
				loading: false,
				privateKey: data.privateKey,
				address: data.address
			}));
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Failed to load wallet';
			this.state.update((s) => ({ ...s, loading: false, error: message }));
		}
	}

	async sign(message: string): Promise<string | null> {
		const { privateKey } = get(this.state);
		if (!privateKey) return null;

		const account = privateKeyToAccount(privateKey as `0x${string}`);
		return account.signMessage({ message });
	}

	getAddress(): string | null {
		return get(this.state).address;
	}
}

export const walletService = new WalletService();
