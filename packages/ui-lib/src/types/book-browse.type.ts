export type { DisplayBook, DisplayBookDetails, DisplayBookAuthor } from 'addons/openlibrary/types';
export { BOOK_SUBJECTS, BOOK_SUBJECT_LABELS, BOOK_FORMAT_OPTIONS } from 'addons/openlibrary/types';
export type { BookSubject, BookFormat } from 'addons/openlibrary/types';

import type { DisplayBook, BookSubject } from 'addons/openlibrary/types';

export interface BookBrowseState {
	searchQuery: string;
	searchResults: DisplayBook[];
	searchPage: number;
	searchTotalPages: number;
	trendingResults: DisplayBook[];
	trendingPage: number;
	trendingTotalPages: number;
	selectedSubject: BookSubject;
	loading: boolean;
	error: string | null;
}
