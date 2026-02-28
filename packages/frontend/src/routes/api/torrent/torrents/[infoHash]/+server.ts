import type { RequestHandler } from './$types';
import { proxyToTorrent } from '$lib/server/torrent-proxy';

export const DELETE: RequestHandler = async ({ params, locals }) => {
	return proxyToTorrent(locals.torrentBaseUrl, `/torrents/${params.infoHash}`, {
		method: 'DELETE'
	});
};
