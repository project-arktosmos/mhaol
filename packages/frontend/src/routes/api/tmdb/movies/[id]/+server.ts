import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { fetchMovie } from 'tmdb';

export const GET: RequestHandler = async ({ params, url, locals }) => {
	const apiKey = locals.tmdbApiKey();
	if (!apiKey) return json({ error: 'TMDB API key not configured' }, { status: 503 });

	const id = parseInt(params.id, 10);
	if (isNaN(id)) return json({ error: 'Invalid movie ID' }, { status: 400 });

	const refresh = url.searchParams.get('refresh') === 'true';
	const cacheRepo = locals.tmdbCacheRepo;

	// Check cache first
	if (!refresh) {
		const cached = cacheRepo.getMovie(id);
		if (cached && cacheRepo.isFresh(cached.fetched_at)) {
			return json(JSON.parse(cached.data));
		}
	}

	try {
		const movie = await fetchMovie(apiKey, id);
		if (!movie) {
			// Graceful degradation: return stale cache if API fails
			const stale = cacheRepo.getMovie(id);
			if (stale) return json(JSON.parse(stale.data));
			return json({ error: 'Movie not found' }, { status: 404 });
		}

		// Cache the result
		cacheRepo.upsertMovie(id, JSON.stringify(movie));
		return json(movie);
	} catch (err) {
		// On API error, try stale cache
		const stale = cacheRepo.getMovie(id);
		if (stale) return json(JSON.parse(stale.data));

		const message = err instanceof Error ? err.message : String(err);
		return json({ error: message }, { status: 500 });
	}
};
