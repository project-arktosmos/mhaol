import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async ({ url, locals }) => {
	const mediaType = url.searchParams.get('mediaType');

	const categories = mediaType
		? locals.categoryRepo.getByMediaType(mediaType)
		: locals.categoryRepo.getAll();

	return json(
		categories.map((c) => ({
			id: c.id,
			mediaTypeId: c.media_type_id,
			label: c.label
		}))
	);
};
