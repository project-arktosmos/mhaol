import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { fetchTvShow } from 'tmdb';

export const GET: RequestHandler = async ({ params, url, locals }) => {
	const apiKey = locals.tmdbApiKey();
	if (!apiKey) return json({ error: 'TMDB API key not configured' }, { status: 503 });

	const id = parseInt(params.id, 10);
	if (isNaN(id)) return json({ error: 'Invalid TV show ID' }, { status: 400 });

	const refresh = url.searchParams.get('refresh') === 'true';
	const cacheRepo = locals.tmdbCacheRepo;

	// Check cache first
	if (!refresh) {
		const cached = cacheRepo.getTvShow(id);
		if (cached && cacheRepo.isFresh(cached.fetched_at)) {
			return json(JSON.parse(cached.data));
		}
	}

	try {
		const tvShow = await fetchTvShow(apiKey, id);
		if (!tvShow) {
			const stale = cacheRepo.getTvShow(id);
			if (stale) return json(JSON.parse(stale.data));
			return json({ error: 'TV show not found' }, { status: 404 });
		}

		cacheRepo.upsertTvShow(id, JSON.stringify(tvShow));
		return json(tvShow);
	} catch (err) {
		const stale = cacheRepo.getTvShow(id);
		if (stale) return json(JSON.parse(stale.data));

		const message = err instanceof Error ? err.message : String(err);
		return json({ error: message }, { status: 500 });
	}
};
