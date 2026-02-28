import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const PUT: RequestHandler = async ({ params, request, locals }) => {
	const item = locals.libraryItemRepo.get(params.itemId);
	if (!item || item.library_id !== params.id) {
		return json({ error: 'Library item not found' }, { status: 404 });
	}

	const body = await request.json();
	const mediaTypeId = body.mediaTypeId;
	if (typeof mediaTypeId !== 'string' || !mediaTypeId.trim()) {
		return json({ error: 'mediaTypeId must be a non-empty string' }, { status: 400 });
	}

	const mediaType = locals.mediaTypeRepo.get(mediaTypeId.trim());
	if (!mediaType) {
		return json({ error: 'Media type not found' }, { status: 404 });
	}

	locals.libraryItemRepo.updateMediaType(params.itemId, mediaTypeId.trim());

	return json({ ok: true });
};
