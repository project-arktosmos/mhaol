import type { RequestHandler } from './$types';
import { proxyToTorrent } from '$lib/server/torrent-proxy';

export const GET: RequestHandler = async ({ locals }) => {
	return proxyToTorrent(locals.torrentBaseUrl, '/torrents');
};

export const POST: RequestHandler = async ({ request, locals }) => {
	const body = await request.text();
	return proxyToTorrent(locals.torrentBaseUrl, '/torrents', {
		method: 'POST',
		body
	});
};
