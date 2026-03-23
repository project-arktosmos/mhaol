import type {
	OpenLibrarySearchDoc,
	OpenLibraryWork,
	OpenLibraryAuthor,
	OpenLibrarySubjectWork,
	DisplayBook,
	DisplayBookDetails,
	DisplayBookAuthor
} from './types.js';

export function getCoverUrl(
	coverId: number | null | undefined,
	size: 'S' | 'M' | 'L' = 'M'
): string | null {
	if (!coverId) return null;
	return `https://covers.openlibrary.org/b/id/${coverId}-${size}.jpg`;
}

export function getAuthorPhotoUrl(photoId: number, size: 'S' | 'M' | 'L' = 'M'): string {
	return `https://covers.openlibrary.org/a/id/${photoId}-${size}.jpg`;
}

function extractWorkKey(key: string): string {
	return key.replace('/works/', '');
}

function extractAuthorKey(key: string): string {
	return key.replace('/authors/', '');
}

function extractDescription(desc: string | { value: string } | undefined): string | null {
	if (!desc) return null;
	if (typeof desc === 'string') return desc;
	return desc.value ?? null;
}

export function searchDocToDisplay(doc: OpenLibrarySearchDoc): DisplayBook {
	return {
		key: extractWorkKey(doc.key),
		title: doc.title,
		authors: doc.author_name ?? [],
		authorKeys: (doc.author_key ?? []).map(extractAuthorKey),
		firstPublishYear: doc.first_publish_year?.toString() ?? '',
		coverId: doc.cover_i ?? null,
		coverUrl: getCoverUrl(doc.cover_i, 'M'),
		subjects: (doc.subject ?? []).slice(0, 10),
		publishers: (doc.publisher ?? []).slice(0, 3),
		pageCount: doc.number_of_pages_median ?? null,
		editionCount: doc.edition_count ?? 0,
		isbn: doc.isbn?.[0] ?? null,
		ratingsAverage: doc.ratings_average ?? null,
		ratingsCount: doc.ratings_count ?? 0
	};
}

export function searchDocsToDisplay(docs: OpenLibrarySearchDoc[]): DisplayBook[] {
	return docs.map(searchDocToDisplay);
}

export function subjectWorkToDisplay(work: OpenLibrarySubjectWork): DisplayBook {
	return {
		key: extractWorkKey(work.key),
		title: work.title,
		authors: work.authors?.map((a) => a.name) ?? [],
		authorKeys: work.authors?.map((a) => extractAuthorKey(a.key)) ?? [],
		firstPublishYear: work.first_publish_year?.toString() ?? '',
		coverId: work.cover_id ?? null,
		coverUrl: getCoverUrl(work.cover_id, 'M'),
		subjects: work.subject?.slice(0, 10) ?? [],
		publishers: [],
		pageCount: null,
		editionCount: work.edition_count ?? 0,
		isbn: null,
		ratingsAverage: null,
		ratingsCount: 0
	};
}

export function subjectWorksToDisplay(works: OpenLibrarySubjectWork[]): DisplayBook[] {
	return works.map(subjectWorkToDisplay);
}

export function workToDisplayDetails(
	work: OpenLibraryWork,
	authors: DisplayBookAuthor[],
	searchDoc?: DisplayBook
): DisplayBookDetails {
	return {
		key: extractWorkKey(work.key),
		title: work.title,
		authors,
		description: extractDescription(work.description),
		covers: work.covers ?? [],
		subjects: (work.subjects ?? []).slice(0, 20),
		firstPublishYear: work.first_publish_date?.split('-')[0] ?? searchDoc?.firstPublishYear ?? '',
		pageCount: searchDoc?.pageCount ?? null,
		isbn: searchDoc?.isbn ?? null,
		coverUrl: getCoverUrl(work.covers?.[0], 'L') ?? searchDoc?.coverUrl ?? null
	};
}

export function authorToDisplay(author: OpenLibraryAuthor): DisplayBookAuthor {
	return {
		key: extractAuthorKey(author.key),
		name: author.name,
		birthDate: author.birth_date ?? null,
		deathDate: author.death_date ?? null,
		bio: extractDescription(author.bio),
		photoUrl: author.photos?.[0] ? getAuthorPhotoUrl(author.photos[0], 'M') : null
	};
}
