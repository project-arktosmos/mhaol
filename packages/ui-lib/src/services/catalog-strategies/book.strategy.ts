import { fetchJson } from 'ui-lib/transport/fetch-helpers';
import { searchDocsToDisplay, subjectWorksToDisplay } from 'addons/openlibrary/transform';
import type {
	OpenLibrarySearchResponse,
	OpenLibrarySubjectResponse
} from 'addons/openlibrary/types';
import type { CatalogItem, CatalogFilterOption } from 'ui-lib/types/catalog.type';
import type { CatalogKindStrategy } from 'ui-lib/services/catalog.service';

const ITEMS_PER_PAGE = 20;

const SUBJECTS: CatalogFilterOption[] = [
	{ id: 'fiction', label: 'Fiction' },
	{ id: 'science_fiction', label: 'Science Fiction' },
	{ id: 'fantasy', label: 'Fantasy' },
	{ id: 'mystery', label: 'Mystery' },
	{ id: 'romance', label: 'Romance' },
	{ id: 'history', label: 'History' },
	{ id: 'science', label: 'Science' },
	{ id: 'philosophy', label: 'Philosophy' },
	{ id: 'biography', label: 'Biography' },
	{ id: 'poetry', label: 'Poetry' }
];

function toBookCatalogItems(
	books: ReturnType<typeof searchDocsToDisplay> | ReturnType<typeof subjectWorksToDisplay>
): CatalogItem[] {
	return books.map((b) => ({
		id: b.key,
		kind: 'book' as const,
		title: b.title,
		sortTitle: b.title.toLowerCase(),
		year: b.firstPublishYear || null,
		overview: null,
		posterUrl: b.coverUrl,
		backdropUrl: null,
		voteAverage: 'ratingsAverage' in b ? (b.ratingsAverage ?? null) : null,
		voteCount: 'ratingsCount' in b ? (b.ratingsCount ?? null) : null,
		parentId: null,
		position: null,
		source: 'openlibrary' as const,
		sourceId: b.key,
		createdAt: '',
		updatedAt: '',
		metadata: {
			openlibraryKey: b.key,
			authors: b.authors,
			authorKeys: 'authorKeys' in b ? b.authorKeys : [],
			firstPublishYear: b.firstPublishYear,
			coverId: 'coverId' in b ? b.coverId : null,
			coverUrl: b.coverUrl,
			subjects: 'subjects' in b ? b.subjects : [],
			publishers: 'publishers' in b ? b.publishers : [],
			pageCount: 'pageCount' in b ? b.pageCount : null,
			editionCount: 'editionCount' in b ? b.editionCount : 0,
			isbn: 'isbn' in b ? b.isbn : null,
			ratingsAverage: 'ratingsAverage' in b ? (b.ratingsAverage ?? null) : null,
			ratingsCount: 'ratingsCount' in b ? (b.ratingsCount ?? 0) : 0,
			description: null,
			authorDetails: []
		}
	}));
}

export const bookStrategy: CatalogKindStrategy = {
	kind: 'book',
	tabs: [{ id: 'trending', label: 'Trending' }],
	filterDefinitions: {
		subject: { label: 'Subject', loadOptions: async () => SUBJECTS }
	},

	async search(query, page, _filters) {
		const data = await fetchJson<OpenLibrarySearchResponse>(
			`/api/openlibrary/search?q=${encodeURIComponent(query)}&page=${page}&limit=${ITEMS_PER_PAGE}`
		);
		const totalPages = Math.ceil((data?.numFound ?? 0) / ITEMS_PER_PAGE);
		return {
			items: toBookCatalogItems(searchDocsToDisplay(data?.docs ?? [])),
			totalPages
		};
	},

	async loadTab(_tabId, page, filters) {
		const subject = filters.subject || 'fiction';
		const data = await fetchJson<OpenLibrarySubjectResponse>(
			`/api/openlibrary/trending/${subject}?page=${page}&limit=${ITEMS_PER_PAGE}`
		);
		const totalPages = Math.ceil((data?.work_count ?? 0) / ITEMS_PER_PAGE);
		return {
			items: toBookCatalogItems(subjectWorksToDisplay(data?.works ?? [])),
			totalPages
		};
	}
};
