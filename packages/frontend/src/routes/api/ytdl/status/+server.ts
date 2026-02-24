import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async ({ locals }) => {
	const stats = locals.downloadManager.getStats();

	if (locals.ytdlp.isAvailable() && !stats.ytdlpVersion) {
		try {
			stats.ytdlpVersion = await locals.ytdlp.getVersion();
		} catch {
			// ignore version fetch errors
		}
	}

	return json(stats);
};
