import { writable, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { apiUrl } from 'ui-lib/lib/api-base';
import { searchDocsToDisplay, subjectWorksToDisplay } from 'addons/openlibrary/transform';
import type { BookBrowseState } from 'ui-lib/types/book-browse.type';
import type { BookSubject } from 'addons/openlibrary/types';
import type {
	OpenLibrarySearchResponse,
	OpenLibrarySubjectResponse
} from 'addons/openlibrary/types';

const ITEMS_PER_PAGE = 20;

const initialState: BookBrowseState = {
	searchQuery: '',
	searchResults: [],
	searchPage: 1,
	searchTotalPages: 1,
	trendingResults: [],
	trendingPage: 1,
	trendingTotalPages: 1,
	selectedSubject: 'fiction',
	loading: false,
	error: null
};

class BookBrowseService {
	public state: Writable<BookBrowseState> = writable(initialState);

	private async fetchJson<T>(path: string): Promise<T> {
		const response = await fetch(apiUrl(path));
		if (!response.ok) throw new Error(`HTTP ${response.status}`);
		return response.json();
	}

	async searchBooks(query: string, page: number = 1): Promise<void> {
		if (!browser) return;
		this.state.update((s) => ({ ...s, loading: true, error: null, searchQuery: query }));

		try {
			const data = await this.fetchJson<OpenLibrarySearchResponse>(
				`/api/openlibrary/search?q=${encodeURIComponent(query)}&page=${page}&limit=${ITEMS_PER_PAGE}`
			);
			const totalPages = Math.ceil((data?.numFound ?? 0) / ITEMS_PER_PAGE);
			this.state.update((s) => ({
				...s,
				searchResults: searchDocsToDisplay(data?.docs ?? []),
				searchPage: page,
				searchTotalPages: totalPages,
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

	async loadTrendingBooks(subject: BookSubject, page: number = 1): Promise<void> {
		if (!browser) return;
		this.state.update((s) => ({ ...s, loading: true, error: null, selectedSubject: subject }));

		try {
			const data = await this.fetchJson<OpenLibrarySubjectResponse>(
				`/api/openlibrary/trending/${subject}?page=${page}&limit=${ITEMS_PER_PAGE}`
			);
			const totalPages = Math.ceil((data?.work_count ?? 0) / ITEMS_PER_PAGE);
			this.state.update((s) => ({
				...s,
				trendingResults: subjectWorksToDisplay(data?.works ?? []),
				trendingPage: page,
				trendingTotalPages: totalPages,
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
}

export const bookBrowseService = new BookBrowseService();
