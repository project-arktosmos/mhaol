import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { fetchSeasonDetails } from 'tmdb';

export const GET: RequestHandler = async ({ params, url, locals }) => {
	const apiKey = locals.tmdbApiKey();
	if (!apiKey) return json({ error: 'TMDB API key not configured' }, { status: 503 });

	const tvShowId = parseInt(params.id, 10);
	const seasonNumber = parseInt(params.seasonNumber, 10);
	if (isNaN(tvShowId) || isNaN(seasonNumber)) {
		return json({ error: 'Invalid TV show ID or season number' }, { status: 400 });
	}

	const refresh = url.searchParams.get('refresh') === 'true';
	const cacheRepo = locals.tmdbCacheRepo;

	// Check cache first
	if (!refresh) {
		const cached = cacheRepo.getSeason(tvShowId, seasonNumber);
		if (cached && cacheRepo.isFresh(cached.fetched_at)) {
			return json(JSON.parse(cached.data));
		}
	}

	try {
		const season = await fetchSeasonDetails(apiKey, tvShowId, seasonNumber);
		if (!season) {
			const stale = cacheRepo.getSeason(tvShowId, seasonNumber);
			if (stale) return json(JSON.parse(stale.data));
			return json({ error: 'Season not found' }, { status: 404 });
		}

		cacheRepo.upsertSeason(tvShowId, seasonNumber, JSON.stringify(season));
		return json(season);
	} catch (err) {
		const stale = cacheRepo.getSeason(tvShowId, seasonNumber);
		if (stale) return json(JSON.parse(stale.data));

		const message = err instanceof Error ? err.message : String(err);
		return json({ error: message }, { status: 500 });
	}
};
