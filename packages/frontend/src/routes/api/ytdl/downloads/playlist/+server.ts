import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const POST: RequestHandler = async ({ request, locals }) => {
	const body = await request.json();

	if (!body.videos || !Array.isArray(body.videos) || body.videos.length === 0) {
		return json({ error: 'Missing or empty videos array' }, { status: 400 });
	}

	const res = await fetch(`${locals.ytdlBaseUrl}/api/downloads/playlist`, {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify(body)
	});

	if (!res.ok) {
		const err = await res.json().catch(() => ({}));
		return json(err, { status: res.status });
	}

	const result = await res.json();

	for (let i = 0; i < result.downloadIds.length; i++) {
		const video = body.videos[i];
		locals.youtubeDownloadRepo.upsert({
			download_id: result.downloadIds[i],
			url: video.url,
			video_id: video.videoId,
			title: video.title,
			state: 'pending',
			progress: 0,
			downloaded_bytes: 0,
			total_bytes: 0,
			output_path: null,
			error: null,
			mode: body.mode ?? 'audio',
			quality: body.quality ?? 'high',
			format: body.format ?? 'aac',
			video_quality: body.videoQuality ?? null,
			video_format: body.videoFormat ?? null,
			thumbnail_url: null,
			duration_seconds: null
		});
	}

	return json(result, { status: 201 });
};
