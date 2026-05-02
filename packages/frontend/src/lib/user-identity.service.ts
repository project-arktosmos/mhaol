import { writable, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { generatePrivateKey, privateKeyToAccount } from 'viem/accounts';

const STORAGE_KEY = 'mhaol-cloud-identity';
const USERNAME_RE = /^[A-Za-z0-9-]+$/;
const USERNAME_MAX = 32;

export interface StoredIdentity {
	address: `0x${string}`;
	privateKey: `0x${string}`;
	username: string;
}

export interface UserDto {
	address: string;
	username: string;
	created_at: string;
	updated_at: string;
	last_login_at: string | null;
}

export interface UserIdentityState {
	loading: boolean;
	identity: StoredIdentity | null;
	user: UserDto | null;
	error: string | null;
}

const initialState: UserIdentityState = {
	loading: true,
	identity: null,
	user: null,
	error: null
};

export class InvalidIdentityError extends Error {}

function randomUsername(): string {
	const chars = 'abcdefghijklmnopqrstuvwxyz0123456789';
	const len = 10;
	const arr = new Uint32Array(len);
	if (typeof crypto !== 'undefined' && crypto.getRandomValues) {
		crypto.getRandomValues(arr);
	}
	let out = 'user-';
	for (let i = 0; i < len; i += 1) out += chars[arr[i] % chars.length];
	return out;
}

function isValidUsername(name: string): boolean {
	return name.length >= 1 && name.length <= USERNAME_MAX && USERNAME_RE.test(name);
}

function buildAuthMessage(): string {
	return `Mhaol Cloud auth at ${new Date().toISOString()}`;
}

async function parseError(res: Response): Promise<string> {
	try {
		const data = await res.json();
		if (data && typeof data.error === 'string') return data.error;
	} catch {
		// fall through
	}
	return `HTTP ${res.status}`;
}

class UserIdentityService {
	state: Writable<UserIdentityState> = writable(initialState);

	private _initialized = false;

	async initialize(): Promise<void> {
		if (!browser || this._initialized) return;
		this._initialized = true;
		this.state.update((s) => ({ ...s, loading: true, error: null }));
		try {
			const identity = this.loadOrCreate();
			const user = await this.loginOrRegister(identity);
			this.state.set({ loading: false, identity, user, error: null });
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Failed to initialize identity';
			this.state.set({ loading: false, identity: null, user: null, error: message });
		}
	}

	exportJson(): string {
		const stored = this.readStored();
		if (!stored) throw new Error('No identity to export');
		return JSON.stringify(stored, null, 2);
	}

	async importJson(raw: string): Promise<void> {
		const parsed = parseStored(raw);
		const account = privateKeyToAccount(parsed.privateKey);
		if (account.address.toLowerCase() !== parsed.address.toLowerCase()) {
			throw new InvalidIdentityError('address does not match private key');
		}
		const identity: StoredIdentity = {
			address: account.address,
			privateKey: parsed.privateKey,
			username: parsed.username
		};
		this.writeStored(identity);
		this.state.update((s) => ({ ...s, loading: true, error: null }));
		const user = await this.loginOrRegister(identity);
		this.state.set({ loading: false, identity, user, error: null });
	}

	async regenerate(): Promise<void> {
		const identity = this.createFresh();
		this.writeStored(identity);
		this.state.update((s) => ({ ...s, loading: true, error: null, user: null }));
		const user = await this.loginOrRegister(identity);
		this.state.set({ loading: false, identity, user, error: null });
	}

	async updateUsername(newUsername: string): Promise<void> {
		const username = newUsername.trim();
		if (!isValidUsername(username)) {
			throw new Error('username must be 1-32 chars of [A-Za-z0-9-]');
		}
		const stored = this.readStored();
		if (!stored) throw new Error('No identity loaded');

		const message = buildAuthMessage();
		const signature = await this.sign(stored.privateKey, message);
		const res = await fetch(`/api/users/${encodeURIComponent(stored.address)}`, {
			method: 'PUT',
			headers: { 'content-type': 'application/json' },
			body: JSON.stringify({ username, message, signature })
		});
		if (!res.ok) throw new Error(await parseError(res));
		const user = (await res.json()) as UserDto;

		const identity: StoredIdentity = { ...stored, username };
		this.writeStored(identity);
		this.state.update((s) => ({ ...s, identity, user, error: null }));
	}

	private loadOrCreate(): StoredIdentity {
		const stored = this.readStored();
		if (stored) return stored;
		const fresh = this.createFresh();
		this.writeStored(fresh);
		return fresh;
	}

	private createFresh(): StoredIdentity {
		const privateKey = generatePrivateKey();
		const account = privateKeyToAccount(privateKey);
		return {
			address: account.address,
			privateKey,
			username: randomUsername()
		};
	}

	private readStored(): StoredIdentity | null {
		if (!browser) return null;
		const raw = localStorage.getItem(STORAGE_KEY);
		if (!raw) return null;
		try {
			return parseStored(raw);
		} catch {
			return null;
		}
	}

	private writeStored(identity: StoredIdentity): void {
		if (!browser) return;
		localStorage.setItem(STORAGE_KEY, JSON.stringify(identity));
	}

	private async sign(privateKey: `0x${string}`, message: string): Promise<string> {
		const account = privateKeyToAccount(privateKey);
		return account.signMessage({ message });
	}

	private async loginOrRegister(identity: StoredIdentity): Promise<UserDto> {
		const message = buildAuthMessage();
		const signature = await this.sign(identity.privateKey, message);

		const loginRes = await fetch('/api/users/login', {
			method: 'POST',
			headers: { 'content-type': 'application/json' },
			body: JSON.stringify({ address: identity.address, message, signature })
		});
		if (loginRes.ok) {
			return (await loginRes.json()) as UserDto;
		}
		if (loginRes.status !== 404) {
			throw new Error(await parseError(loginRes));
		}

		// Not registered yet — auto-register with a fresh signature so the
		// timestamp stays inside the freshness window.
		const registerMessage = buildAuthMessage();
		const registerSignature = await this.sign(identity.privateKey, registerMessage);
		const regRes = await fetch('/api/users/register', {
			method: 'POST',
			headers: { 'content-type': 'application/json' },
			body: JSON.stringify({
				address: identity.address,
				username: identity.username,
				message: registerMessage,
				signature: registerSignature
			})
		});
		if (!regRes.ok) {
			// Username collision — try once with a freshly randomized name so
			// the auto-onboard never blocks on a clash with a stale local entry.
			if (regRes.status === 409) {
				const newUsername = randomUsername();
				const retryMessage = buildAuthMessage();
				const retrySignature = await this.sign(identity.privateKey, retryMessage);
				const retry = await fetch('/api/users/register', {
					method: 'POST',
					headers: { 'content-type': 'application/json' },
					body: JSON.stringify({
						address: identity.address,
						username: newUsername,
						message: retryMessage,
						signature: retrySignature
					})
				});
				if (!retry.ok) throw new Error(await parseError(retry));
				identity.username = newUsername;
				this.writeStored(identity);
				return (await retry.json()) as UserDto;
			}
			throw new Error(await parseError(regRes));
		}
		return (await regRes.json()) as UserDto;
	}
}

function parseStored(raw: string): StoredIdentity {
	let parsed: unknown;
	try {
		parsed = JSON.parse(raw);
	} catch (e) {
		throw new InvalidIdentityError(`invalid JSON: ${e instanceof Error ? e.message : e}`);
	}
	if (!parsed || typeof parsed !== 'object') {
		throw new InvalidIdentityError('expected an object');
	}
	const obj = parsed as Record<string, unknown>;
	const address = obj.address;
	const privateKey = obj.privateKey;
	const username = obj.username;
	if (typeof address !== 'string' || !/^0x[0-9a-fA-F]{40}$/.test(address)) {
		throw new InvalidIdentityError('invalid address');
	}
	if (typeof privateKey !== 'string' || !/^0x[0-9a-fA-F]{64}$/.test(privateKey)) {
		throw new InvalidIdentityError('invalid privateKey');
	}
	if (typeof username !== 'string' || !isValidUsername(username)) {
		throw new InvalidIdentityError('invalid username');
	}
	return {
		address: address as `0x${string}`,
		privateKey: privateKey as `0x${string}`,
		username
	};
}

export const userIdentityService = new UserIdentityService();
export const USERNAME_PATTERN = USERNAME_RE;
