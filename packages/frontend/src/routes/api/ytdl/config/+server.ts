import type { RequestHandler } from './$types';
import { proxyToYtdl } from '$lib/server/ytdl-proxy';

export const GET: RequestHandler = async ({ locals }) => {
	return proxyToYtdl(locals.ytdlBaseUrl, '/api/config');
};

export const PUT: RequestHandler = async ({ request, locals }) => {
	const body = await request.text();
	return proxyToYtdl(locals.ytdlBaseUrl, '/api/config', {
		method: 'PUT',
		body
	});
};
