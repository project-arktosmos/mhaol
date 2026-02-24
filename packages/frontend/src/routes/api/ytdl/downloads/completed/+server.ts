import type { RequestHandler } from './$types';
import { proxyToYtdl } from '$lib/server/ytdl-proxy';

export const DELETE: RequestHandler = async ({ locals }) => {
	const res = await proxyToYtdl(locals.ytdlBaseUrl, '/api/downloads/completed', {
		method: 'DELETE'
	});
	locals.youtubeDownloadRepo.deleteByStates(['completed', 'failed', 'cancelled']);
	return res;
};
