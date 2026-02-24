import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async ({ params, locals }) => {
	const progress = locals.downloadManager.getProgress(params.id);
	if (!progress) return json({ error: 'Download not found' }, { status: 404 });
	return json(progress);
};

export const DELETE: RequestHandler = async ({ params, locals }) => {
	locals.downloadManager.cancelDownload(params.id);
	return json({ ok: true });
};
