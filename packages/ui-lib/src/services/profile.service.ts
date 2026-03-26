import { writable, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { fetchRaw } from 'ui-lib/transport/fetch-helpers';
import type {
	UserProfile,
	RemoteProfile,
	ProfileState,
	ProfileDetail
} from 'ui-lib/types/profile.type';
import { clientIdentityService } from 'ui-lib/services/client-identity.service';
import { generateRandomUsername } from 'ui-lib/utils/random-username';

const LOCAL_STORAGE_KEY = 'user-profile-username';

const emptyProfile: UserProfile = { username: '', wallet: '' };

const initialState: ProfileState = {
	loading: false,
	local: emptyProfile,
	remoteProfiles: [],
	error: null
};

class ProfileService {
	public state: Writable<ProfileState> = writable(initialState);

	private _initialized = false;

	async initialize(): Promise<void> {
		if (!browser || this._initialized) return;
		this._initialized = true;

		// Clean up stale key from earlier implementation
		localStorage.removeItem('user-profile');

		const identity = clientIdentityService.loadLocal();
		const savedUsername = this.readUsername();
		const local: UserProfile = {
			wallet: identity.address,
			username: savedUsername || identity.name || generateRandomUsername()
		};

		this.state.update((s) => ({ ...s, local }));
		await this.refreshRemote();
	}

	updateUsername(username: string): void {
		this.writeUsername(username);
		this.state.update((s) => ({
			...s,
			local: { ...s.local, username }
		}));
	}

	async shareWithNode(): Promise<void> {
		let local: UserProfile = emptyProfile;
		this.state.subscribe((s) => {
			local = s.local;
		})();

		if (!local.username && !local.wallet) return;

		await fetchRaw('/api/profiles', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify(local)
		});
		await this.refreshRemote();
	}

	async refreshRemote(): Promise<void> {
		this.state.update((s) => ({ ...s, loading: true, error: null }));
		try {
			const res = await fetchRaw('/api/profiles');
			if (!res.ok) throw new Error(`HTTP ${res.status}`);
			const raw: { username: string; wallet: string; added_at: string }[] = await res.json();

			const countsPromises = raw.map(async (p) => {
				try {
					const countRes = await fetchRaw(
						`/api/favorites/count?wallet=${encodeURIComponent(p.wallet)}`
					);
					if (countRes.ok) {
						const data: { count: number } = await countRes.json();
						return data.count;
					}
				} catch {
					// ignore
				}
				return 0;
			});
			const counts = await Promise.all(countsPromises);

			const remoteProfiles: RemoteProfile[] = raw.map((p, i) => ({
				...p,
				favoriteCount: counts[i]
			}));

			this.state.update((s) => ({ ...s, loading: false, remoteProfiles }));
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Failed to load profiles';
			this.state.update((s) => ({ ...s, loading: false, error: message }));
		}
	}

	async fetchProfile(wallet: string): Promise<ProfileDetail> {
		const res = await fetchRaw(`/api/profiles/${encodeURIComponent(wallet)}`);
		if (!res.ok) throw new Error(`HTTP ${res.status}`);
		const data = await res.json();
		const countRes = await fetchRaw(`/api/favorites/count?wallet=${encodeURIComponent(wallet)}`);
		const countData = countRes.ok ? await countRes.json() : { count: 0 };
		return {
			profile: {
				...data.profile,
				favoriteCount: countData.count
			},
			favorites: data.favorites
		};
	}

	private readUsername(): string {
		try {
			const raw = localStorage.getItem(LOCAL_STORAGE_KEY);
			if (raw) return raw;
		} catch {
			// ignore
		}
		return '';
	}

	private writeUsername(username: string): void {
		localStorage.setItem(LOCAL_STORAGE_KEY, username);
	}
}

export const profileService = new ProfileService();
