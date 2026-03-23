// Raw Open Library API types

export interface OpenLibrarySearchDoc {
	key: string; // e.g. "/works/OL12345W"
	title: string;
	author_name?: string[];
	author_key?: string[];
	first_publish_year?: number;
	cover_i?: number;
	isbn?: string[];
	subject?: string[];
	publisher?: string[];
	language?: string[];
	number_of_pages_median?: number;
	edition_count?: number;
	ratings_average?: number;
	ratings_count?: number;
}

export interface OpenLibrarySearchResponse {
	numFound: number;
	start: number;
	docs: OpenLibrarySearchDoc[];
}

export interface OpenLibraryWork {
	key: string;
	title: string;
	description?: string | { value: string };
	covers?: number[];
	subjects?: string[];
	subject_places?: string[];
	subject_times?: string[];
	authors?: Array<{
		author: { key: string };
		type?: { key: string };
	}>;
	first_publish_date?: string;
	created?: { value: string };
}

export interface OpenLibraryAuthor {
	key: string;
	name: string;
	birth_date?: string;
	death_date?: string;
	bio?: string | { value: string };
	photos?: number[];
	alternate_names?: string[];
	wikipedia?: string;
}

export interface OpenLibrarySubjectResponse {
	name: string;
	work_count: number;
	works: OpenLibrarySubjectWork[];
}

export interface OpenLibrarySubjectWork {
	key: string;
	title: string;
	cover_id?: number;
	cover_edition_key?: string;
	authors: Array<{ name: string; key: string }>;
	first_publish_year?: number;
	edition_count?: number;
	subject?: string[];
}

// Display types (camelCase, transformed for UI)

export interface DisplayBook {
	key: string;
	title: string;
	authors: string[];
	authorKeys: string[];
	firstPublishYear: string;
	coverId: number | null;
	coverUrl: string | null;
	subjects: string[];
	publishers: string[];
	pageCount: number | null;
	editionCount: number;
	isbn: string | null;
	ratingsAverage: number | null;
	ratingsCount: number;
}

export interface DisplayBookDetails {
	key: string;
	title: string;
	authors: DisplayBookAuthor[];
	description: string | null;
	covers: number[];
	subjects: string[];
	firstPublishYear: string;
	pageCount: number | null;
	isbn: string | null;
	coverUrl: string | null;
}

export interface DisplayBookAuthor {
	key: string;
	name: string;
	birthDate: string | null;
	deathDate: string | null;
	bio: string | null;
	photoUrl: string | null;
}

export const BOOK_SUBJECTS = [
	'fiction',
	'non-fiction',
	'science_fiction',
	'fantasy',
	'mystery',
	'romance',
	'history',
	'science',
	'biography',
	'philosophy',
	'poetry',
	'horror',
	'thriller',
	'children'
] as const;

export type BookSubject = (typeof BOOK_SUBJECTS)[number];

export const BOOK_SUBJECT_LABELS: Record<BookSubject, string> = {
	fiction: 'Fiction',
	'non-fiction': 'Non-Fiction',
	science_fiction: 'Sci-Fi',
	fantasy: 'Fantasy',
	mystery: 'Mystery',
	romance: 'Romance',
	history: 'History',
	science: 'Science',
	biography: 'Biography',
	philosophy: 'Philosophy',
	poetry: 'Poetry',
	horror: 'Horror',
	thriller: 'Thriller',
	children: 'Children'
};

export const BOOK_FORMAT_OPTIONS = ['EPUB', 'PDF', 'MOBI', 'AZW3'] as const;
export type BookFormat = (typeof BOOK_FORMAT_OPTIONS)[number];
