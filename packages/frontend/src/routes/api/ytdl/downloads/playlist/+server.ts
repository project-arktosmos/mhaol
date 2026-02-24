import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const POST: RequestHandler = async ({ request, locals }) => {
	const body = await request.json();

	if (!body.videos || !Array.isArray(body.videos) || body.videos.length === 0) {
		return json({ error: 'Missing or empty videos array' }, { status: 400 });
	}

	const downloadIds = locals.downloadManager.queuePlaylistDownloads({
		videos: body.videos,
		mode: body.mode,
		quality: body.quality,
		format: body.format,
		videoQuality: body.videoQuality,
		videoFormat: body.videoFormat
	});

	return json({ downloadIds }, { status: 201 });
};
