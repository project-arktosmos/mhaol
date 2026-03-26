import { writable, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { generatePrivateKey, privateKeyToAccount } from 'viem/accounts';
import { hashMessage } from 'viem';
import { generateRandomUsername } from 'ui-lib/utils/random-username';
import type { PassportData } from 'webrtc/types';

const STORAGE_KEY = 'client-identity';

interface StoredIdentity {
	name: string;
	privateKey: `0x${string}`;
	username?: string;
	profilePictureUrl?: string;
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
	private _signalingUrl = '';

	async initialize(signalingUrl: string): Promise<void> {
		if (!browser || this._initialized) return;
		this._initialized = true;
		this._signalingUrl = signalingUrl;

		try {
			const { name, privateKey, username, profilePictureUrl } = this.loadOrCreateStored();

			const identity = await this.buildIdentity(name, privateKey, username, profilePictureUrl);
			this.state.set({ loading: false, identity, error: null });
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Failed to initialize identity';
			this.state.set({ loading: false, identity: null, error: message });
		}
	}

	async signMessage(message: string): Promise<string> {
		if (!browser) throw new Error('Not in browser');
		const stored = localStorage.getItem(STORAGE_KEY);
		if (!stored) throw new Error('No identity initialized');
		const { privateKey }: StoredIdentity = JSON.parse(stored);
		const account = privateKeyToAccount(privateKey);
		return account.signMessage({ message });
	}

	async regenerate(): Promise<void> {
		if (!browser) return;
		const name = generateRandomUsername();
		const privateKey = generatePrivateKey();
		const username = name;
		localStorage.setItem(STORAGE_KEY, JSON.stringify({ name, privateKey, username }));
		const identity = await this.buildIdentity(name, privateKey, username);
		this.state.set({ loading: false, identity, error: null });
	}

	/** Load or create the stored keypair without full initialization (no signalingUrl needed). */
	loadLocal(): { name: string; address: string; username?: string; profilePictureUrl?: string } {
		if (!browser) return { name: '', address: '' };
		const { name, privateKey, username, profilePictureUrl } = this.loadOrCreateStored();
		const account = privateKeyToAccount(privateKey);
		return { name, address: account.address, username, profilePictureUrl };
	}

	/** Update the stored display name. */
	updateName(newName: string): void {
		if (!browser) return;
		const stored = localStorage.getItem(STORAGE_KEY);
		if (!stored) return;
		const parsed: StoredIdentity = JSON.parse(stored);
		parsed.name = newName;
		localStorage.setItem(STORAGE_KEY, JSON.stringify(parsed));
	}

	/** Update the profile (username + optional profile picture) and rebuild the passport. */
	async updateProfile(username: string, profilePictureUrl?: string): Promise<void> {
		if (!browser) return;
		const stored = localStorage.getItem(STORAGE_KEY);
		if (!stored) return;
		const parsed: StoredIdentity = JSON.parse(stored);
		parsed.username = username;
		parsed.profilePictureUrl = profilePictureUrl;
		localStorage.setItem(STORAGE_KEY, JSON.stringify(parsed));
		const identity = await this.buildIdentity(parsed.name, parsed.privateKey, username, profilePictureUrl);
		this.state.set({ loading: false, identity, error: null });
	}

	private loadOrCreateStored(): StoredIdentity {
		const stored = localStorage.getItem(STORAGE_KEY);
		if (stored) {
			return JSON.parse(stored);
		}
		const identity: StoredIdentity = {
			name: generateRandomUsername(),
			privateKey: generatePrivateKey()
		};
		localStorage.setItem(STORAGE_KEY, JSON.stringify(identity));
		return identity;
	}

	private async buildIdentity(
		name: string,
		privateKey: `0x${string}`,
		username?: string,
		profilePictureUrl?: string
	): Promise<{ name: string; address: string; passport: PassportData }> {
		const account = privateKeyToAccount(privateKey);
		const payload: Record<string, string> = {
			name,
			address: account.address,
			instanceType: 'client',
			signalingUrl: this._signalingUrl
		};
		if (username) payload.username = username;
		if (profilePictureUrl) payload.profilePictureUrl = profilePictureUrl;
		const raw = JSON.stringify(payload);
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
