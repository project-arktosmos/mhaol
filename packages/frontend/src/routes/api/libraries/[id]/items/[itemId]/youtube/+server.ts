import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const PUT: RequestHandler = async ({ params, request, locals }) => {
	const item = locals.libraryItemRepo.get(params.itemId);
	if (!item || item.library_id !== params.id) {
		return json({ error: 'Library item not found' }, { status: 404 });
	}

	const body = await request.json();
	const youtubeId = body.youtubeId;
	if (typeof youtubeId !== 'string' || !youtubeId.trim()) {
		return json({ error: 'youtubeId must be a non-empty string' }, { status: 400 });
	}

	locals.libraryItemLinkRepo.upsert({
		id: crypto.randomUUID(),
		library_item_id: params.itemId,
		service: 'youtube',
		service_id: youtubeId.trim(),
		season_number: null,
		episode_number: null
	});

	return json({ ok: true });
};

export const DELETE: RequestHandler = async ({ params, locals }) => {
	const item = locals.libraryItemRepo.get(params.itemId);
	if (!item || item.library_id !== params.id) {
		return json({ error: 'Library item not found' }, { status: 404 });
	}

	locals.libraryItemLinkRepo.delete(params.itemId, 'youtube');

	return json({ ok: true });
};
