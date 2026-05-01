import { writable, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { fetchRaw } from '$transport/fetch-helpers';
import type { UserProfile, RemoteProfile, ProfileState, ProfileDetail } from '$types/profile.type';
import { clientIdentityService } from '$services/client-identity.service';
import { generateRandomUsername } from '$utils/random-username';

const OLD_USERNAME_KEY = 'user-profile-username';

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

		// Clean up stale keys from earlier implementations
		localStorage.removeItem('user-profile');

		const identity = clientIdentityService.loadLocal();

		// Migrate old separate username storage into client identity
		const oldUsername = localStorage.getItem(OLD_USERNAME_KEY);
		if (oldUsername && !identity.username) {
			await clientIdentityService.updateProfile(oldUsername);
			localStorage.removeItem(OLD_USERNAME_KEY);
		} else {
			localStorage.removeItem(OLD_USERNAME_KEY);
		}

		// Re-read after potential migration
		const updated = clientIdentityService.loadLocal();
		const username = updated.username || updated.name || generateRandomUsername();

		// If there was no username stored yet, persist it
		if (!updated.username) {
			await clientIdentityService.updateProfile(username);
		}

		const local: UserProfile = {
			wallet: updated.address,
			username,
			profilePictureUrl: updated.profilePictureUrl
		};

		this.state.update((s) => ({ ...s, local }));
		await this.refreshRemote();
	}

	updateUsername(username: string): void {
		clientIdentityService.updateProfile(username);
		this.state.update((s) => ({
			...s,
			local: { ...s.local, username }
		}));
	}

	updateProfilePicture(profilePictureUrl: string | undefined): void {
		let currentUsername = '';
		this.state.subscribe((s) => {
			currentUsername = s.local.username;
		})();
		clientIdentityService.updateProfile(currentUsername, profilePictureUrl);
		this.state.update((s) => ({
			...s,
			local: { ...s.local, profilePictureUrl }
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
			body: JSON.stringify({
				username: local.username,
				wallet: local.wallet,
				profilePictureUrl: local.profilePictureUrl
			})
		});
		await this.refreshRemote();
	}

	async refreshRemote(): Promise<void> {
		this.state.update((s) => ({ ...s, loading: true, error: null }));
		try {
			const res = await fetchRaw('/api/profiles');
			if (!res.ok) throw new Error(`HTTP ${res.status}`);
			const raw: {
				username: string;
				wallet: string;
				profile_picture_url?: string;
				added_at: string;
			}[] = await res.json();

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
}

export const profileService = new ProfileService();
