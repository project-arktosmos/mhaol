import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { searchTvShows } from 'tmdb';

export const GET: RequestHandler = async ({ url, locals }) => {
	const apiKey = locals.tmdbApiKey();
	if (!apiKey) return json({ error: 'TMDB API key not configured' }, { status: 503 });

	const query = url.searchParams.get('q');
	if (!query?.trim()) return json({ error: 'Missing q parameter' }, { status: 400 });

	const page = parseInt(url.searchParams.get('page') ?? '1', 10) || 1;
	const yearParam = url.searchParams.get('year');
	const year = yearParam ? parseInt(yearParam, 10) || undefined : undefined;

	try {
		const result = await searchTvShows(apiKey, query, page, year);
		if (!result) return json({ error: 'No results' }, { status: 404 });
		return json(result);
	} catch (err) {
		const message = err instanceof Error ? err.message : String(err);
		return json({ error: message }, { status: 500 });
	}
};
