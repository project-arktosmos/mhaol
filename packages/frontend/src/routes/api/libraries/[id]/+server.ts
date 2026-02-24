import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const DELETE: RequestHandler = async ({ params, locals }) => {
	const deleted = locals.libraryRepo.delete(params.id);
	if (!deleted) {
		return json({ error: 'Library not found' }, { status: 404 });
	}
	return json({ ok: true });
};
