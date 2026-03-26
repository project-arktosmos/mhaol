import { writable, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { fetchRaw } from 'ui-lib/transport/fetch-helpers';
import type { Favorite, FavoritesState } from 'ui-lib/types/favorite.type';

const LOCAL_STORAGE_KEY = 'user-favorites';

const initialState: FavoritesState = {
	loading: false,
	items: [],
	error: null
};

function mapFromApi(raw: Record<string, unknown>): Favorite {
	return {
		id: raw.id as string,
		wallet: raw.wallet as string,
		service: raw.service as string,
		serviceId: raw.service_id as string,
		label: raw.label as string,
		createdAt: raw.created_at as string
	};
}

class FavoritesService {
	public state: Writable<FavoritesState> = writable(initialState);

	private _initialized = false;
	private _wallet = '';

	async initialize(wallet: string): Promise<void> {
		if (!browser || !wallet) return;
		this._wallet = wallet;
		this._initialized = true;

		const local = this.readLocal();
		this.state.update((s) => ({ ...s, items: local }));

		await this.refresh();
	}

	async refresh(): Promise<void> {
		if (!this._wallet) return;
		this.state.update((s) => ({ ...s, loading: true, error: null }));
		try {
			const res = await fetchRaw(`/api/favorites?wallet=${encodeURIComponent(this._wallet)}`);
			if (!res.ok) throw new Error(`HTTP ${res.status}`);
			const raw: Record<string, unknown>[] = await res.json();
			const items = raw.map(mapFromApi);
			this.state.update((s) => ({ ...s, loading: false, items }));
			this.writeLocal(items);
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Failed to load favorites';
			this.state.update((s) => ({ ...s, loading: false, error: message }));
		}
	}

	async add(service: string, serviceId: string, label: string): Promise<void> {
		const tempItem: Favorite = {
			id: crypto.randomUUID(),
			wallet: this._wallet,
			service,
			serviceId,
			label,
			createdAt: new Date().toISOString()
		};
		this.state.update((s) => {
			const items = [...s.items, tempItem];
			this.writeLocal(items);
			return { ...s, items };
		});

		await fetchRaw('/api/favorites', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ wallet: this._wallet, service, serviceId, label })
		});
		await this.refresh();
	}

	async remove(service: string, serviceId: string): Promise<void> {
		this.state.update((s) => {
			const items = s.items.filter((f) => !(f.service === service && f.serviceId === serviceId));
			this.writeLocal(items);
			return { ...s, items };
		});

		await fetchRaw('/api/favorites', {
			method: 'DELETE',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ wallet: this._wallet, service, serviceId })
		});
		await this.refresh();
	}

	isFavorite(service: string, serviceId: string): boolean {
		let found = false;
		this.state.subscribe((s) => {
			found = s.items.some((f) => f.service === service && f.serviceId === serviceId);
		})();
		return found;
	}

	async toggle(service: string, serviceId: string, label: string): Promise<void> {
		if (this.isFavorite(service, serviceId)) {
			await this.remove(service, serviceId);
		} else {
			await this.add(service, serviceId, label);
		}
	}

	private readLocal(): Favorite[] {
		try {
			const raw = localStorage.getItem(LOCAL_STORAGE_KEY);
			if (raw) return JSON.parse(raw);
		} catch {
			// ignore
		}
		return [];
	}

	private writeLocal(items: Favorite[]): void {
		localStorage.setItem(LOCAL_STORAGE_KEY, JSON.stringify(items));
	}
}

export const favoritesService = new FavoritesService();
