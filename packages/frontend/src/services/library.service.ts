import { writable, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { ArrayServiceClass } from '$services/classes/array-service.class';
import type { Library, DirectoryEntry, BrowseDirectoryResponse } from '$types/library.type';
import { type MediaType } from '$types/library.type';

export interface LibraryServiceState {
	showAddForm: boolean;
	browsing: boolean;
	browseError: string | null;
	currentBrowsePath: string;
	browseDirectories: DirectoryEntry[];
	browseParent: string | null;
	selectedPath: string;
	selectedName: string;
	selectedMediaTypes: MediaType[];
}

const initialState: LibraryServiceState = {
	showAddForm: false,
	browsing: false,
	browseError: null,
	currentBrowsePath: '',
	browseDirectories: [],
	browseParent: null,
	selectedPath: '',
	selectedName: '',
	selectedMediaTypes: []
};

class LibraryService extends ArrayServiceClass<Library> {
	public state: Writable<LibraryServiceState> = writable(initialState);

	constructor() {
		super('libraries', []);
	}

	async browseDirectory(path?: string): Promise<void> {
		if (!browser) return;

		this.state.update((s) => ({ ...s, browsing: true, browseError: null }));

		try {
			const params = path ? `?path=${encodeURIComponent(path)}` : '';
			const response = await this.fetchJson<BrowseDirectoryResponse>(
				`/api/libraries/browse${params}`
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

	addLibrary(name: string, path: string, mediaTypes: MediaType[]): Library {
		const library: Library = {
			id: crypto.randomUUID(),
			name,
			path,
			mediaTypes,
			dateAdded: Date.now()
		};
		this.add(library);
		this.resetForm();
		return library;
	}

	removeLibrary(library: Library): void {
		this.remove(library);
	}

	openAddForm(): void {
		this.state.update((s) => ({
			...s,
			showAddForm: true,
			selectedPath: '',
			selectedName: '',
			selectedMediaTypes: [],
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
			selectedMediaTypes: [],
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

	toggleMediaType(mediaType: MediaType): void {
		this.state.update((s) => {
			const types = s.selectedMediaTypes.includes(mediaType)
				? s.selectedMediaTypes.filter((t) => t !== mediaType)
				: [...s.selectedMediaTypes, mediaType];
			return { ...s, selectedMediaTypes: types };
		});
	}

	private resetForm(): void {
		this.state.update((s) => ({
			...s,
			showAddForm: false,
			selectedPath: '',
			selectedName: '',
			selectedMediaTypes: [],
			currentBrowsePath: '',
			browseDirectories: [],
			browseParent: null,
			browseError: null
		}));
	}

	private async fetchJson<T>(path: string, init?: RequestInit): Promise<T> {
		const response = await fetch(path, {
			...init,
			headers: {
				'Content-Type': 'application/json',
				...init?.headers
			}
		});

		if (!response.ok) {
			const body = await response.json().catch(() => ({}));
			throw new Error((body as { error?: string }).error ?? `HTTP ${response.status}`);
		}

		return response.json() as Promise<T>;
	}
}

export const libraryService = new LibraryService();
