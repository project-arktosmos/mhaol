import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async ({ url, locals }) => {
	const playlistUrl = url.searchParams.get('url');
	if (!playlistUrl) return json({ error: 'Missing url parameter' }, { status: 400 });

	if (!locals.ytdlp.isAvailable()) {
		return json({ error: 'yt-dlp is not available. Please install it first.' }, { status: 503 });
	}

	try {
		const info = await locals.ytdlp.getPlaylistInfo(playlistUrl);
		return json(info);
	} catch (err) {
		return json({ error: err instanceof Error ? err.message : String(err) }, { status: 500 });
	}
};
