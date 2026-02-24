import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async ({ locals }) => {
	const config = locals.torrentManager.getConfig();
	return json({ download_path: config.downloadPath });
};

export const PUT: RequestHandler = async ({ request, locals }) => {
	const body = await request.json();
	if (body.download_path) {
		locals.torrentManager.updateConfig({ downloadPath: body.download_path });
	}
	const config = locals.torrentManager.getConfig();
	return json({ download_path: config.downloadPath });
};
