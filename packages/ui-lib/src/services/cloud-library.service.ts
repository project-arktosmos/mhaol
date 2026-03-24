import { writable, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { fetchJson } from 'ui-lib/transport/fetch-helpers';
import type {
	CloudLibrary,
	CloudItem,
	DirectoryEntry,
	BrowseDirectoryResponse,
	CloudScanResponse
} from 'ui-lib/types/cloud.type';

export interface CloudLibraryServiceState {
	items: Record<string, CloudItem[]>;
	itemsLoading: Record<string, boolean>;
	browsing: boolean;
	browseError: string | null;
	currentBrowsePath: string;
	browseDirectories: DirectoryEntry[];
	browseParent: string | null;
	showAddForm: boolean;
	selectedPath: string;
	selectedName: string;
}

const initialState: CloudLibraryServiceState = {
	items: {},
	itemsLoading: {},
	browsing: false,
	browseError: null,
	currentBrowsePath: '',
	browseDirectories: [],
	browseParent: null,
	showAddForm: false,
	selectedPath: '',
	selectedName: ''
};

class CloudLibraryService {
	public store: Writable<CloudLibrary[]> = writable([]);
	public state: Writable<CloudLibraryServiceState> = writable(initialState);

	private initialized = false;

	async initialize(): Promise<void> {
		if (!browser || this.initialized) return;

		try {
			const libraries = await fetchJson<CloudLibrary[]>('/api/cloud/libraries');
			this.store.set(libraries);
			this.initialized = true;

			for (const library of libraries) {
				this.fetchItems(library.id);
			}
		} catch (error) {
			console.error('[cloud-library] Failed to initialize:', error);
		}
	}

	async addLibrary(name: string, path: string): Promise<void> {
		if (!browser) return;

		try {
			const library = await fetchJson<CloudLibrary>('/api/cloud/libraries', {
				method: 'POST',
				body: JSON.stringify({ name, path, kind: 'filesystem' })
			});
			this.store.update((libs) => [...libs, library]);
			this.closeAddForm();
		} catch (error) {
			console.error('[cloud-library] Failed to add library:', error);
		}
	}

	async removeLibrary(id: string): Promise<void> {
		if (!browser) return;

		try {
			await fetchJson(`/api/cloud/libraries/${id}`, { method: 'DELETE' });
			this.store.update((libs) => libs.filter((l) => l.id !== id));
			this.state.update((s) => {
				const { [id]: _, ...rest } = s.items;
				return { ...s, items: rest };
			});
		} catch (error) {
			console.error('[cloud-library] Failed to remove library:', error);
		}
	}

	async scanLibrary(id: string): Promise<void> {
		if (!browser) return;

		this.state.update((s) => ({
			...s,
			itemsLoading: { ...s.itemsLoading, [id]: true }
		}));

		try {
			const response = await fetchJson<CloudScanResponse>(`/api/cloud/libraries/${id}/scan`, {
				method: 'POST'
			});
			this.state.update((s) => ({
				...s,
				items: { ...s.items, [id]: response.items },
				itemsLoading: { ...s.itemsLoading, [id]: false }
			}));
			this.store.update((libs) =>
				libs.map((l) =>
					l.id === id ? { ...l, itemCount: response.itemCount, scanStatus: 'idle' as const } : l
				)
			);
		} catch (error) {
			console.error('[cloud-library] Failed to scan library:', error);
			this.state.update((s) => ({
				...s,
				itemsLoading: { ...s.itemsLoading, [id]: false }
			}));
		}
	}

	async fetchItems(libraryId: string): Promise<void> {
		if (!browser) return;

		this.state.update((s) => ({
			...s,
			itemsLoading: { ...s.itemsLoading, [libraryId]: true }
		}));

		try {
			const items = await fetchJson<CloudItem[]>(`/api/cloud/libraries/${libraryId}/items`);
			this.state.update((s) => ({
				...s,
				items: { ...s.items, [libraryId]: items },
				itemsLoading: { ...s.itemsLoading, [libraryId]: false }
			}));
		} catch (error) {
			console.error('[cloud-library] Failed to fetch items:', error);
			this.state.update((s) => ({
				...s,
				itemsLoading: { ...s.itemsLoading, [libraryId]: false }
			}));
		}
	}

	async browseDirectory(path?: string): Promise<void> {
		if (!browser) return;

		this.state.update((s) => ({ ...s, browsing: true, browseError: null }));

		try {
			const params = path ? `?path=${encodeURIComponent(path)}` : '';
			const response = await fetchJson<BrowseDirectoryResponse>(
				`/api/cloud/libraries/browse${params}`
			);
			this.state.update((s) => ({
				...s,
				browsing: false,
				currentBrowsePath: response.path,
				browseDirectories: response.directories,
				browseParent: response.parent
			}));
		} catch (error) {
			const errorMsg = error instanceof Error ? error.message : String(error);
			this.state.update((s) => ({
				...s,
				browsing: false,
				browseError: `Failed to browse directory: ${errorMsg}`
			}));
		}
	}

	openAddForm(): void {
		this.state.update((s) => ({
			...s,
			showAddForm: true,
			selectedPath: '',
			selectedName: '',
			browseError: null
		}));
		this.browseDirectory();
	}

	closeAddForm(): void {
		this.state.update((s) => ({
			...s,
			showAddForm: false,
			selectedPath: '',
			selectedName: '',
			currentBrowsePath: '',
			browseDirectories: [],
			browseParent: null,
			browseError: null
		}));
	}

	selectDirectory(path: string, name: string): void {
		this.state.update((s) => ({
			...s,
			selectedPath: path,
			selectedName: s.selectedName || name
		}));
	}

	setSelectedName(name: string): void {
		this.state.update((s) => ({ ...s, selectedName: name }));
	}

}

export const cloudLibraryService = new CloudLibraryService();
