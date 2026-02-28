import type { RequestHandler } from './$types';
import { proxyToTorrent } from '$lib/server/torrent-proxy';

export const GET: RequestHandler = async ({ locals }) => {
	return proxyToTorrent(locals.torrentBaseUrl, '/config');
};

export const PUT: RequestHandler = async ({ request, locals }) => {
	const body = await request.text();
	return proxyToTorrent(locals.torrentBaseUrl, '/config', {
		method: 'PUT',
		body
	});
};
