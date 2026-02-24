import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const DELETE: RequestHandler = async ({ locals }) => {
	locals.downloadManager.clearQueue();
	return json({ ok: true });
};
