import type { RequestHandler } from './$types';

export const GET: RequestHandler = async ({ locals }) => {
	const res = await fetch(`${locals.torrentBaseUrl}/torrents/events`, {
		headers: { Accept: 'text/event-stream' }
	}).catch(() => null);

	if (!res?.ok || !res.body) {
		return new Response('Torrent server not available', { status: 503 });
	}

	return new Response(res.body, {
		headers: {
			'Content-Type': 'text/event-stream',
			'Cache-Control': 'no-cache',
			Connection: 'keep-alive'
		}
	});
};
