import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const POST: RequestHandler = async ({ locals }) => {
	locals.torrentManager.clearStorage();
	locals.torrentDownloadRepo.deleteAll();
	return json({ ok: true });
};
