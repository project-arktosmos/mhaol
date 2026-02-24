import type { RequestHandler } from './$types';
import { proxyToYtdl } from '$lib/server/ytdl-proxy';

export const GET: RequestHandler = async ({ locals }) => {
	return proxyToYtdl(locals.ytdlBaseUrl, '/api/status');
};
