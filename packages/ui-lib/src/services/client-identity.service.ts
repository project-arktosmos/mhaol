import { writable, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { generatePrivateKey, privateKeyToAccount } from 'viem/accounts';
import { hashMessage } from 'viem';
import type { PassportData } from 'webrtc/types';

const STORAGE_KEY = 'client-identity';

interface StoredIdentity {
	name: string;
	privateKey: `0x${string}`;
}

interface ClientIdentityState {
	loading: boolean;
	identity: { name: string; address: string; passport: PassportData } | null;
	error: string | null;
}

const initialState: ClientIdentityState = {
	loading: true,
	identity: null,
	error: null
};

class ClientIdentityService {
	public state: Writable<ClientIdentityState> = writable(initialState);

	private _initialized = false;

	async initialize(): Promise<void> {
		if (!browser || this._initialized) return;
		this._initialized = true;

		try {
			const stored = localStorage.getItem(STORAGE_KEY);
			let name: string;
			let privateKey: `0x${string}`;

			if (stored) {
				const parsed: StoredIdentity = JSON.parse(stored);
				name = parsed.name;
				privateKey = parsed.privateKey;
			} else {
				name = 'default';
				privateKey = generatePrivateKey();
				localStorage.setItem(STORAGE_KEY, JSON.stringify({ name, privateKey }));
			}

			const identity = await this.buildIdentity(name, privateKey);
			this.state.set({ loading: false, identity, error: null });
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Failed to initialize identity';
			this.state.set({ loading: false, identity: null, error: message });
		}
	}

	async regenerate(): Promise<void> {
		if (!browser) return;
		const name = 'default';
		const privateKey = generatePrivateKey();
		localStorage.setItem(STORAGE_KEY, JSON.stringify({ name, privateKey }));
		const identity = await this.buildIdentity(name, privateKey);
		this.state.set({ loading: false, identity, error: null });
	}

	private async buildIdentity(
		name: string,
		privateKey: `0x${string}`
	): Promise<{ name: string; address: string; passport: PassportData }> {
		const account = privateKeyToAccount(privateKey);
		const raw = JSON.stringify({ name, address: account.address, instanceType: 'client' });
		const signature = await account.signMessage({ message: raw });
		const hash = hashMessage(raw);
		return {
			name,
			address: account.address,
			passport: { raw, hash, signature }
		};
	}
}

export const clientIdentityService = new ClientIdentityService();
