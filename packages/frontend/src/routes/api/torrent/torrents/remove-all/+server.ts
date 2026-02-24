import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const POST: RequestHandler = async ({ locals }) => {
	const count = await locals.torrentManager.removeAll();
	return json({ removed: count });
};
