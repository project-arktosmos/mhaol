import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async ({ locals }) => {
	return json({
		initialized: locals.torrentManager.isInitialized(),
		download_path: locals.torrentManager.getConfig().downloadPath,
		stats: locals.torrentManager.stats()
	});
};
