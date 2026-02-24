import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async ({ locals }) => {
	return json(locals.downloadManager.getAllProgress());
};

export const POST: RequestHandler = async ({ request, locals }) => {
	const body = await request.json();

	if (!body.url || !body.videoId || !body.title) {
		return json({ error: 'Missing required fields: url, videoId, title' }, { status: 400 });
	}

	const downloadId = locals.downloadManager.queueDownload({
		url: body.url,
		videoId: body.videoId,
		title: body.title,
		mode: body.mode,
		quality: body.quality,
		format: body.format,
		videoQuality: body.videoQuality,
		videoFormat: body.videoFormat
	});

	return json({ downloadId }, { status: 201 });
};
