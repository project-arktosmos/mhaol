import type { RequestHandler } from './$types';
import { proxyToTorrent } from '$lib/server/torrent-proxy';

export const POST: RequestHandler = async ({ params, locals }) => {
	return proxyToTorrent(locals.torrentBaseUrl, `/torrents/${params.infoHash}/resume`, {
		method: 'POST'
	});
};
