import { writable, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import type {
	Library,
	DirectoryEntry,
	BrowseDirectoryResponse,
	LibraryFile,
	LibraryFilesResponse
} from '$types/library.type';
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
	expandedLibraryId: string | null;
	libraryFiles: Record<string, LibraryFile[]>;
	libraryFilesLoading: Record<string, boolean>;
	libraryFilesError: Record<string, string | null>;
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
	selectedMediaTypes: [],
	expandedLibraryId: null,
	libraryFiles: {},
	libraryFilesLoading: {},
	libraryFilesError: {}
};

class LibraryService {
	public store: Writable<Library[]> = writable([]);
	public state: Writable<LibraryServiceState> = writable(initialState);

	private initialized = false;

	async initialize(): Promise<void> {
		if (!browser || this.initialized) return;

		try {
			const libraries = await this.fetchJson<Library[]>('/api/libraries');
			this.store.set(libraries);
			this.initialized = true;
		} catch (error) {
			console.error('[library] Failed to initialize:', error);
		}
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

	async addLibrary(name: string, path: string, mediaTypes: MediaType[]): Promise<void> {
		if (!browser) return;

		try {
			const library = await this.fetchJson<Library>('/api/libraries', {
				method: 'POST',
				body: JSON.stringify({ name, path, mediaTypes })
			});

			this.store.update((items) => [...items, library]);
			this.resetForm();
		} catch (error) {
			console.error('[library] Failed to add library:', error);
		}
	}

	async removeLibrary(library: Library): Promise<void> {
		if (!browser) return;

		try {
			await this.fetchJson(`/api/libraries/${library.id}`, { method: 'DELETE' });
			this.store.update((items) => items.filter((i) => i.id !== library.id));
		} catch (error) {
			console.error('[library] Failed to remove library:', error);
		}
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

	async toggleLibraryFiles(libraryId: string): Promise<void> {
		let currentExpanded: string | null = null;
		const unsubscribe = this.state.subscribe((s) => {
			currentExpanded = s.expandedLibraryId;
		});
		unsubscribe();

		if (currentExpanded === libraryId) {
			this.state.update((s) => ({ ...s, expandedLibraryId: null }));
			return;
		}

		this.state.update((s) => ({ ...s, expandedLibraryId: libraryId }));
		await this.fetchLibraryFiles(libraryId);
	}

	async fetchLibraryFiles(libraryId: string): Promise<void> {
		if (!browser) return;

		this.state.update((s) => ({
			...s,
			libraryFilesLoading: { ...s.libraryFilesLoading, [libraryId]: true },
			libraryFilesError: { ...s.libraryFilesError, [libraryId]: null }
		}));

		try {
			const response = await this.fetchJson<LibraryFilesResponse>(
				`/api/libraries/${libraryId}/files`
			);
			this.state.update((s) => ({
				...s,
				libraryFiles: { ...s.libraryFiles, [libraryId]: response.files },
				libraryFilesLoading: { ...s.libraryFilesLoading, [libraryId]: false }
			}));
		} catch (error) {
			const errorMsg = error instanceof Error ? error.message : String(error);
			this.state.update((s) => ({
				...s,
				libraryFilesLoading: { ...s.libraryFilesLoading, [libraryId]: false },
				libraryFilesError: { ...s.libraryFilesError, [libraryId]: errorMsg }
			}));
		}
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
