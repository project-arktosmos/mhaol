import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const DELETE: RequestHandler = async ({ locals }) => {
	locals.downloadManager.clearCompleted();
	return json({ ok: true });
};
