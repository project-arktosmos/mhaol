import { writable, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import type {
	CatalogItem,
	CatalogKind,
	CatalogBrowseState,
	CatalogTab,
	CatalogFilterOption
} from 'ui-lib/types/catalog.type';

export interface CatalogKindStrategy {
	kind: CatalogKind;
	pinService: string;
	tabs: CatalogTab[];
	filterDefinitions: Record<
		string,
		{ label: string; loadOptions: () => Promise<CatalogFilterOption[]> }
	>;
	search(
		query: string,
		page: number,
		filters: Record<string, string>
	): Promise<{ items: CatalogItem[]; totalPages: number }>;
	loadTab(
		tabId: string,
		page: number,
		filters: Record<string, string>
	): Promise<{ items: CatalogItem[]; totalPages: number }>;
	resolveByIds?(ids: string[]): Promise<CatalogItem[]>;
}

const initialState: CatalogBrowseState = {
	kind: 'movie',
	items: [],
	loading: false,
	error: null,
	searchQuery: '',
	page: 1,
	totalPages: 1,
	activeTab: '',
	tabs: [],
	filters: {},
	filterOptions: {}
};

class CatalogService {
	public state: Writable<CatalogBrowseState> = writable(initialState);
	private strategies = new Map<CatalogKind, CatalogKindStrategy>();
	private currentStrategy: CatalogKindStrategy | null = null;

	registerStrategy(strategy: CatalogKindStrategy): void {
		this.strategies.set(strategy.kind, strategy);
	}

	async activate(kind: CatalogKind): Promise<void> {
		if (!browser) return;
		const strategy = this.strategies.get(kind);
		if (!strategy) return;

		this.currentStrategy = strategy;
		this.state.set({
			...initialState,
			kind,
			tabs: strategy.tabs,
			activeTab: strategy.tabs[0]?.id ?? ''
		});

		const filterOptions: Record<string, CatalogFilterOption[]> = {};
		for (const [id, def] of Object.entries(strategy.filterDefinitions)) {
			filterOptions[id] = await def.loadOptions();
		}
		this.state.update((s) => ({ ...s, filterOptions }));

		if (strategy.tabs.length > 0) {
			await this.loadTab(strategy.tabs[0].id);
		}
	}

	async search(query: string, page: number = 1): Promise<void> {
		if (!this.currentStrategy) return;
		this.state.update((s) => ({
			...s,
			loading: true,
			error: null,
			searchQuery: query,
			page,
			activeTab: '__search__'
		}));

		try {
			let filters: Record<string, string> = {};
			this.state.subscribe((s) => (filters = s.filters))();
			const result = await this.currentStrategy.search(query, page, filters);
			this.state.update((s) => ({
				...s,
				items: result.items,
				totalPages: result.totalPages,
				loading: false
			}));
		} catch (error) {
			this.state.update((s) => ({
				...s,
				loading: false,
				error: error instanceof Error ? error.message : String(error)
			}));
		}
	}

	async loadTab(tabId: string, page: number = 1): Promise<void> {
		if (!this.currentStrategy) return;
		this.state.update((s) => ({
			...s,
			loading: true,
			error: null,
			activeTab: tabId,
			page,
			searchQuery: ''
		}));

		try {
			let filters: Record<string, string> = {};
			this.state.subscribe((s) => (filters = s.filters))();
			const result = await this.currentStrategy.loadTab(tabId, page, filters);
			this.state.update((s) => ({
				...s,
				items: result.items,
				totalPages: result.totalPages,
				loading: false
			}));
		} catch (error) {
			this.state.update((s) => ({
				...s,
				loading: false,
				error: error instanceof Error ? error.message : String(error)
			}));
		}
	}

	async setFilter(filterId: string, value: string): Promise<void> {
		this.state.update((s) => ({
			...s,
			filters: { ...s.filters, [filterId]: value }
		}));

		let state: CatalogBrowseState = initialState;
		this.state.subscribe((s) => (state = s))();

		if (state.activeTab === '__search__' && state.searchQuery) {
			await this.search(state.searchQuery, 1);
		} else {
			await this.loadTab(state.activeTab, 1);
		}
	}

	async loadPage(page: number): Promise<void> {
		let state: CatalogBrowseState = initialState;
		this.state.subscribe((s) => (state = s))();

		if (state.activeTab === '__search__' && state.searchQuery) {
			await this.search(state.searchQuery, page);
		} else {
			await this.loadTab(state.activeTab, page);
		}
	}
}

export const catalogService = new CatalogService();
