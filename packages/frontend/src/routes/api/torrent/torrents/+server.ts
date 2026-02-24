import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async ({ locals }) => {
	return json(locals.torrentManager.list());
};

export const POST: RequestHandler = async ({ request, locals }) => {
	const body = await request.json();

	if (!body.source) {
		return json({ error: 'Missing required field: source' }, { status: 400 });
	}

	try {
		const info = await locals.torrentManager.add({
			source: body.source,
			downloadPath: body.downloadPath,
			paused: body.paused
		});
		return json(info, { status: 201 });
	} catch (err) {
		const message = err instanceof Error ? err.message : String(err);
		return json({ error: message }, { status: 400 });
	}
};
