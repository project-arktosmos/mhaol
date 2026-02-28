import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const PUT: RequestHandler = async ({ params, request, locals }) => {
	const item = locals.libraryItemRepo.get(params.itemId);
	if (!item || item.library_id !== params.id) {
		return json({ error: 'Library item not found' }, { status: 404 });
	}

	const body = await request.json();
	const tmdbId = body.tmdbId;
	if (typeof tmdbId !== 'number') {
		return json({ error: 'tmdbId must be a number' }, { status: 400 });
	}

	const seasonNumber = typeof body.seasonNumber === 'number' ? body.seasonNumber : null;
	const episodeNumber = typeof body.episodeNumber === 'number' ? body.episodeNumber : null;

	locals.libraryItemLinkRepo.upsert({
		id: crypto.randomUUID(),
		library_item_id: params.itemId,
		service: 'tmdb',
		service_id: String(tmdbId),
		season_number: seasonNumber,
		episode_number: episodeNumber
	});

	return json({ ok: true });
};

export const DELETE: RequestHandler = async ({ params, locals }) => {
	const item = locals.libraryItemRepo.get(params.itemId);
	if (!item || item.library_id !== params.id) {
		return json({ error: 'Library item not found' }, { status: 404 });
	}

	locals.libraryItemLinkRepo.delete(params.itemId, 'tmdb');

	return json({ ok: true });
};
