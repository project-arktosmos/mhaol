import { fetchJson } from 'ui-lib/transport/fetch-helpers';
import { searchDocsToDisplay, subjectWorksToDisplay, getCoverUrl } from 'addons/openlibrary/transform';
import type {
	OpenLibrarySearchResponse,
	OpenLibrarySubjectResponse,
	OpenLibraryWork
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

function extractWorkKey(key: string): string {
	return key.replace(/^\/works\//, '');
}

function workToCatalogItem(work: OpenLibraryWork): CatalogItem {
	const key = extractWorkKey(work.key);
	const coverId = work.covers?.[0] ?? null;
	return {
		id: key,
		kind: 'book' as const,
		title: work.title,
		sortTitle: work.title.toLowerCase(),
		year: work.first_publish_date?.split('-')[0] ?? null,
		overview: null,
		posterUrl: getCoverUrl(coverId, 'M'),
		backdropUrl: null,
		voteAverage: null,
		voteCount: null,
		parentId: null,
		position: null,
		source: 'openlibrary' as const,
		sourceId: key,
		createdAt: '',
		updatedAt: '',
		metadata: {
			openlibraryKey: key,
			authors: [],
			authorKeys: work.authors?.map((a) => a.author.key.replace(/^\/authors\//, '')) ?? [],
			firstPublishYear: work.first_publish_date?.split('-')[0] ?? '',
			coverId,
			coverUrl: getCoverUrl(coverId, 'M'),
			subjects: (work.subjects ?? []).slice(0, 10),
			publishers: [],
			pageCount: null,
			editionCount: 0,
			isbn: null,
			ratingsAverage: null,
			ratingsCount: 0,
			description: null,
			authorDetails: []
		}
	};
}

export const bookStrategy: CatalogKindStrategy = {
	kind: 'book',
	pinService: 'openlibrary',
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
	},

	async resolveByIds(ids) {
		const results = await Promise.allSettled(
			ids.map((id) => fetchJson<OpenLibraryWork>(`/api/openlibrary/works/${id}`))
		);
		return results
			.filter(
				(r): r is PromiseFulfilledResult<OpenLibraryWork> =>
					r.status === 'fulfilled' && r.value != null
			)
			.map((r) => workToCatalogItem(r.value));
	}
};
