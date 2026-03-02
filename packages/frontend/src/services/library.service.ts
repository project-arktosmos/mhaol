import { get, writable, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { apiUrl } from '$lib/api-base';
import type {
	Library,
	DirectoryEntry,
	BrowseDirectoryResponse,
	LibraryFile,
	LibraryFilesResponse,
	MediaTypeOption,
	CategoryOption
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

			for (const library of libraries) {
				this.fetchLibraryFiles(library.id as string);
			}
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

	async scanLibraryFiles(libraryId: string): Promise<void> {
		if (!browser) return;

		this.state.update((s) => ({
			...s,
			libraryFilesLoading: { ...s.libraryFilesLoading, [libraryId]: true },
			libraryFilesError: { ...s.libraryFilesError, [libraryId]: null }
		}));

		try {
			const response = await this.fetchJson<LibraryFilesResponse>(
				`/api/libraries/${libraryId}/scan`,
				{ method: 'POST' }
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

	async scanAllLibraries(): Promise<void> {
		if (!browser) return;
		const libraries = get(this.store);
		await Promise.all(libraries.map((lib) => this.scanLibraryFiles(lib.id as string)));
	}

	async linkTmdb(
		libraryId: string,
		itemId: string,
		tmdbId: number,
		seasonNumber: number | null = null,
		episodeNumber: number | null = null
	): Promise<void> {
		if (!browser) return;

		await this.fetchJson(`/api/libraries/${libraryId}/items/${itemId}/tmdb`, {
			method: 'PUT',
			body: JSON.stringify({ tmdbId, seasonNumber, episodeNumber })
		});

		this.state.update((s) => {
			const files = s.libraryFiles[libraryId];
			if (!files) return s;
			return {
				...s,
				libraryFiles: {
					...s.libraryFiles,
					[libraryId]: files.map((f) =>
						f.id === itemId
							? {
									...f,
									links: {
										...f.links,
										tmdb: { serviceId: String(tmdbId), seasonNumber, episodeNumber }
									}
								}
							: f
					)
				}
			};
		});
	}

	async unlinkTmdb(libraryId: string, itemId: string): Promise<void> {
		if (!browser) return;

		await this.fetchJson(`/api/libraries/${libraryId}/items/${itemId}/tmdb`, {
			method: 'DELETE'
		});

		this.state.update((s) => {
			const files = s.libraryFiles[libraryId];
			if (!files) return s;
			return {
				...s,
				libraryFiles: {
					...s.libraryFiles,
					[libraryId]: files.map((f) => {
						if (f.id !== itemId) return f;
						const { tmdb: _, ...rest } = f.links;
						return { ...f, links: rest };
					})
				}
			};
		});
	}

	async linkYoutube(libraryId: string, itemId: string, youtubeId: string): Promise<void> {
		if (!browser) return;

		await this.fetchJson(`/api/libraries/${libraryId}/items/${itemId}/youtube`, {
			method: 'PUT',
			body: JSON.stringify({ youtubeId })
		});

		this.state.update((s) => {
			const files = s.libraryFiles[libraryId];
			if (!files) return s;
			return {
				...s,
				libraryFiles: {
					...s.libraryFiles,
					[libraryId]: files.map((f) =>
						f.id === itemId
							? {
									...f,
									links: {
										...f.links,
										youtube: { serviceId: youtubeId, seasonNumber: null, episodeNumber: null }
									}
								}
							: f
					)
				}
			};
		});
	}

	async unlinkYoutube(libraryId: string, itemId: string): Promise<void> {
		if (!browser) return;

		await this.fetchJson(`/api/libraries/${libraryId}/items/${itemId}/youtube`, {
			method: 'DELETE'
		});

		this.state.update((s) => {
			const files = s.libraryFiles[libraryId];
			if (!files) return s;
			return {
				...s,
				libraryFiles: {
					...s.libraryFiles,
					[libraryId]: files.map((f) => {
						if (f.id !== itemId) return f;
						const { youtube: _, ...rest } = f.links;
						return { ...f, links: rest };
					})
				}
			};
		});
	}

	async linkMusicBrainz(libraryId: string, itemId: string, musicbrainzId: string): Promise<void> {
		if (!browser) return;

		await this.fetchJson(`/api/libraries/${libraryId}/items/${itemId}/musicbrainz`, {
			method: 'PUT',
			body: JSON.stringify({ musicbrainzId })
		});

		this.state.update((s) => {
			const files = s.libraryFiles[libraryId];
			if (!files) return s;
			return {
				...s,
				libraryFiles: {
					...s.libraryFiles,
					[libraryId]: files.map((f) =>
						f.id === itemId
							? {
									...f,
									links: {
										...f.links,
										musicbrainz: {
											serviceId: musicbrainzId,
											seasonNumber: null,
											episodeNumber: null
										}
									}
								}
							: f
					)
				}
			};
		});
	}

	async unlinkMusicBrainz(libraryId: string, itemId: string): Promise<void> {
		if (!browser) return;

		await this.fetchJson(`/api/libraries/${libraryId}/items/${itemId}/musicbrainz`, {
			method: 'DELETE'
		});

		this.state.update((s) => {
			const files = s.libraryFiles[libraryId];
			if (!files) return s;
			return {
				...s,
				libraryFiles: {
					...s.libraryFiles,
					[libraryId]: files.map((f) => {
						if (f.id !== itemId) return f;
						const { musicbrainz: _, ...rest } = f.links;
						return { ...f, links: rest };
					})
				}
			};
		});
	}

	async updateCategory(libraryId: string, itemId: string, categoryId: string): Promise<void> {
		if (!browser) return;

		await this.fetchJson(`/api/libraries/${libraryId}/items/${itemId}/category`, {
			method: 'PUT',
			body: JSON.stringify({ categoryId })
		});

		this.state.update((s) => {
			const files = s.libraryFiles[libraryId];
			if (!files) return s;
			return {
				...s,
				libraryFiles: {
					...s.libraryFiles,
					[libraryId]: files.map((f) => (f.id === itemId ? { ...f, categoryId } : f))
				}
			};
		});
	}

	async clearCategory(libraryId: string, itemId: string): Promise<void> {
		if (!browser) return;

		await this.fetchJson(`/api/libraries/${libraryId}/items/${itemId}/category`, {
			method: 'DELETE'
		});

		this.state.update((s) => {
			const files = s.libraryFiles[libraryId];
			if (!files) return s;
			return {
				...s,
				libraryFiles: {
					...s.libraryFiles,
					[libraryId]: files.map((f) => (f.id === itemId ? { ...f, categoryId: null } : f))
				}
			};
		});
	}

	async updateMediaType(libraryId: string, itemId: string, mediaTypeId: string): Promise<void> {
		if (!browser) return;

		await this.fetchJson(`/api/libraries/${libraryId}/items/${itemId}/media-type`, {
			method: 'PUT',
			body: JSON.stringify({ mediaTypeId })
		});

		this.state.update((s) => {
			const files = s.libraryFiles[libraryId];
			if (!files) return s;
			return {
				...s,
				libraryFiles: {
					...s.libraryFiles,
					[libraryId]: files.map((f) =>
						f.id === itemId ? { ...f, mediaType: mediaTypeId as MediaType, categoryId: null } : f
					)
				}
			};
		});
	}

	async fetchMediaTypes(): Promise<MediaTypeOption[]> {
		return this.fetchJson<MediaTypeOption[]>('/api/libraries/media-types');
	}

	async fetchCategories(mediaType?: string): Promise<CategoryOption[]> {
		const params = mediaType ? `?mediaType=${encodeURIComponent(mediaType)}` : '';
		return this.fetchJson<CategoryOption[]>(`/api/libraries/categories${params}`);
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
		const response = await fetch(apiUrl(path), {
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
