import type { RequestHandler } from './$types';
import { proxyToYtdl } from '$lib/server/ytdl-proxy';

export const DELETE: RequestHandler = async ({ locals }) => {
	const res = await proxyToYtdl(locals.ytdlBaseUrl, '/api/downloads/queue', {
		method: 'DELETE'
	});
	locals.youtubeDownloadRepo.deleteByStates(['pending']);
	return res;
};
