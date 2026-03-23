import type {
	OpenLibrarySearchResponse,
	OpenLibraryWork,
	OpenLibraryAuthor,
	OpenLibrarySubjectResponse
} from './types.js';
import { RateLimiter } from '../../common/src/rate-limiter.js';

const BASE_URL = 'https://openlibrary.org';
const HEADERS = {
	Accept: 'application/json',
	'User-Agent': 'Mhaol/0.0.1 (https://github.com/project-arktosmos/mhaol)'
};

const rateLimiter = new RateLimiter(1, 3); // Open Library asks for max 1 req/sec

async function olFetch<T>(url: string): Promise<T | null> {
	return rateLimiter.enqueue(async () => {
		const response = await fetch(url, { headers: HEADERS });
		if (!response.ok) {
			if (response.status === 404) return null;
			if (response.status === 429) throw new Error('429 Rate Limited');
			return null;
		}
		return await response.json();
	});
}

export async function searchBooks(
	query: string,
	page: number = 1,
	limit: number = 20
): Promise<OpenLibrarySearchResponse | null> {
	const params = new URLSearchParams({
		q: query,
		page: page.toString(),
		limit: limit.toString(),
		fields:
			'key,title,author_name,author_key,first_publish_year,cover_i,isbn,subject,publisher,language,number_of_pages_median,edition_count,ratings_average,ratings_count'
	});
	return olFetch<OpenLibrarySearchResponse>(`${BASE_URL}/search.json?${params}`);
}

export async function getWork(workKey: string): Promise<OpenLibraryWork | null> {
	const key = workKey.startsWith('/works/') ? workKey : `/works/${workKey}`;
	return olFetch<OpenLibraryWork>(`${BASE_URL}${key}.json`);
}

export async function getAuthor(authorKey: string): Promise<OpenLibraryAuthor | null> {
	const key = authorKey.startsWith('/authors/') ? authorKey : `/authors/${authorKey}`;
	return olFetch<OpenLibraryAuthor>(`${BASE_URL}${key}.json`);
}

export async function getSubjectBooks(
	subject: string,
	limit: number = 20,
	offset: number = 0
): Promise<OpenLibrarySubjectResponse | null> {
	const params = new URLSearchParams({
		limit: limit.toString(),
		offset: offset.toString()
	});
	return olFetch<OpenLibrarySubjectResponse>(`${BASE_URL}/subjects/${subject}.json?${params}`);
}
