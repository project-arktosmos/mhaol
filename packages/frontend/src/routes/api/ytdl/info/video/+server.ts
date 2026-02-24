import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { proxyToYtdl } from '$lib/server/ytdl-proxy';

export const GET: RequestHandler = async ({ url, locals }) => {
	const videoUrl = url.searchParams.get('url');
	if (!videoUrl) return json({ error: 'Missing url parameter' }, { status: 400 });

	return proxyToYtdl(locals.ytdlBaseUrl, `/api/info/video?url=${encodeURIComponent(videoUrl)}`);
};
