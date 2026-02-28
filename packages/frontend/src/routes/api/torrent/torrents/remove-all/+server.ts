import type { RequestHandler } from './$types';
import { proxyToTorrent } from '$lib/server/torrent-proxy';

export const POST: RequestHandler = async ({ locals }) => {
	return proxyToTorrent(locals.torrentBaseUrl, '/torrents/remove-all', {
		method: 'POST'
	});
};
