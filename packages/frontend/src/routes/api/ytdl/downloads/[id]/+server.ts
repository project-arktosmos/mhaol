import type { RequestHandler } from './$types';
import { proxyToYtdl } from '$lib/server/ytdl-proxy';

export const GET: RequestHandler = async ({ params, locals }) => {
	return proxyToYtdl(locals.ytdlBaseUrl, `/api/downloads/${params.id}`);
};

export const DELETE: RequestHandler = async ({ params, locals }) => {
	const res = await proxyToYtdl(locals.ytdlBaseUrl, `/api/downloads/${params.id}`, {
		method: 'DELETE'
	});
	locals.youtubeDownloadRepo.delete(params.id);
	return res;
};
