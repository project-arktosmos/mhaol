import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async ({ locals }) => {
	const mediaTypes = locals.mediaTypeRepo.getAll();
	return json(mediaTypes.map((mt) => ({ id: mt.id, label: mt.label })));
};
