import type { OMDBDetails } from './types.js';

const OMDB_BASE_URL = 'https://www.omdbapi.com';

async function omdbFetch(
	apiKey: string,
	params: Record<string, string>
): Promise<OMDBDetails | null> {
	const searchParams = new URLSearchParams({ ...params, apikey: apiKey });
	const url = `${OMDB_BASE_URL}/?${searchParams.toString()}`;

	const response = await fetch(url, { headers: { Accept: 'application/json' } });
	if (!response.ok) return null;

	const payload = (await response.json()) as OMDBDetails;
	if (payload.Response !== 'True') return null;
	return payload;
}

/** Look up an item by IMDb id (`tt0111161`-style). The plot field is
 *  intentionally not requested — OMDb only ships ratings + the small set
 *  of metadata fields we project. */
export async function getByImdbId(apiKey: string, imdbId: string): Promise<OMDBDetails | null> {
	if (!imdbId) return null;
	return omdbFetch(apiKey, { i: imdbId });
}

/** Title-based fallback for items we have no IMDb id for. `year` narrows
 *  the match for re-released / re-made titles. */
export async function getByTitle(
	apiKey: string,
	title: string,
	year?: number
): Promise<OMDBDetails | null> {
	if (!title) return null;
	const params: Record<string, string> = { t: title };
	if (year) params.y = year.toString();
	return omdbFetch(apiKey, params);
}
