import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { getNowPlaying, getPopular, getUpcoming, getTopRated } from 'tmdb';

const categoryFetchers = {
	now_playing: getNowPlaying,
	popular: getPopular,
	upcoming: getUpcoming,
	top_rated: getTopRated
} as const;

type MovieCategory = keyof typeof categoryFetchers;

export const GET: RequestHandler = async ({ url, locals }) => {
	const apiKey = locals.tmdbApiKey();
	if (!apiKey) return json({ error: 'TMDB API key not configured' }, { status: 503 });

	const category = url.searchParams.get('category') as MovieCategory | null;
	if (!category || !(category in categoryFetchers)) {
		return json({ error: 'Invalid category. Use: now_playing, popular, upcoming, top_rated' }, { status: 400 });
	}

	const page = parseInt(url.searchParams.get('page') ?? '1', 10) || 1;

	try {
		const result = await categoryFetchers[category](apiKey, page);
		if (!result) return json({ error: 'No results' }, { status: 404 });
		return json(result);
	} catch (err) {
		const message = err instanceof Error ? err.message : String(err);
		return json({ error: message }, { status: 500 });
	}
};
