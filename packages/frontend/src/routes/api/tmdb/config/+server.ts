import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async ({ locals }) => {
	const apiKey = locals.tmdbApiKey();
	return json({ configured: !!apiKey });
};
