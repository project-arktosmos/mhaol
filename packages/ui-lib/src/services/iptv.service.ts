import { writable, get, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { fetchJson, resolveApiUrl } from 'ui-lib/transport/fetch-helpers';
import type {
	IptvServiceState,
	IptvCategory,
	IptvCountry,
	IptvSearchResult,
	IptvChannelDetail,
	IptvEpgResponse
} from 'ui-lib/types/iptv.type';

const API_PREFIX = '/api/iptv';

const initialState: IptvServiceState = {
	initialized: false,
	loading: false,
	error: null,
	channels: [],
	total: 0,
	page: 1,
	categories: [],
	countries: [],
	query: '',
	selectedCategory: '',
	selectedCountry: '',
	epgOnly: false
};

class IptvService {
	public state: Writable<IptvServiceState> = writable(initialState);

	private _initialized = false;

	async initialize(): Promise<void> {
		if (!browser || this._initialized) return;
		this._initialized = true;

		this.state.update((s) => ({ ...s, initialized: true, loading: true }));

		try {
			const [categories, countries] = await Promise.all([
				fetchJson<IptvCategory[]>(`${API_PREFIX}/categories`),
				fetchJson<IptvCountry[]>(`${API_PREFIX}/countries`)
			]);

			this.state.update((s) => ({
				...s,
				categories,
				countries,
				loading: false
			}));

			await this.search();
		} catch (err) {
			this.state.update((s) => ({
				...s,
				loading: false,
				error: err instanceof Error ? err.message : String(err)
			}));
		}
	}

	async search(page = 1): Promise<void> {
		const current = get(this.state);
		this.state.update((s) => ({ ...s, loading: true, error: null, page }));

		const params = new URLSearchParams();
		if (current.query) params.set('q', current.query);
		if (current.selectedCategory) params.set('category', current.selectedCategory);
		if (current.selectedCountry) params.set('country', current.selectedCountry);
		if (current.epgOnly) params.set('hasEpg', 'true');
		params.set('page', String(page));
		params.set('limit', '50');

		try {
			const result = await fetchJson<IptvSearchResult>(
				`${API_PREFIX}/channels?${params.toString()}`
			);
			this.state.update((s) => ({
				...s,
				channels: result.channels,
				total: result.total,
				page: result.page,
				loading: false
			}));
		} catch (err) {
			this.state.update((s) => ({
				...s,
				loading: false,
				error: err instanceof Error ? err.message : String(err)
			}));
		}
	}

	async getChannel(id: string): Promise<IptvChannelDetail | null> {
		try {
			return await fetchJson<IptvChannelDetail>(
				`${API_PREFIX}/channel?id=${encodeURIComponent(id)}`
			);
		} catch {
			return null;
		}
	}

	getStreamUrl(channelId: string): string {
		return resolveApiUrl(`${API_PREFIX}/stream?id=${encodeURIComponent(channelId)}`);
	}

	async getEpg(channelId: string): Promise<IptvEpgResponse | null> {
		try {
			return await fetchJson<IptvEpgResponse>(
				`${API_PREFIX}/epg?id=${encodeURIComponent(channelId)}`
			);
		} catch {
			return null;
		}
	}

	setQuery(query: string): void {
		this.state.update((s) => ({ ...s, query }));
	}

	setCategory(category: string): void {
		this.state.update((s) => ({ ...s, selectedCategory: category }));
	}

	setCountry(country: string): void {
		this.state.update((s) => ({ ...s, selectedCountry: country }));
	}

	setEpgOnly(epgOnly: boolean): void {
		this.state.update((s) => ({ ...s, epgOnly }));
	}
}

export const iptvService = new IptvService();
