import type { RequestHandler } from './$types';
import { proxyToTorrent } from '$lib/server/torrent-proxy';

export const GET: RequestHandler = async ({ locals }) => {
	return proxyToTorrent(locals.torrentBaseUrl, '/status');
};
