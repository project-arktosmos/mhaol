import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async ({ locals }) => {
	return json(locals.downloadManager.getConfig());
};

export const PUT: RequestHandler = async ({ request, locals }) => {
	const body = await request.json();
	locals.downloadManager.updateConfig(body);
	return json(locals.downloadManager.getConfig());
};
