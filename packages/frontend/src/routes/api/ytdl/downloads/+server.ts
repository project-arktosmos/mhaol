import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { proxyToYtdl } from '$lib/server/ytdl-proxy';

export const GET: RequestHandler = async ({ locals }) => {
	return proxyToYtdl(locals.ytdlBaseUrl, '/api/downloads');
};

export const POST: RequestHandler = async ({ request, locals }) => {
	const body = await request.json();

	if (!body.url || !body.videoId || !body.title) {
		return json({ error: 'Missing required fields: url, videoId, title' }, { status: 400 });
	}

	const res = await fetch(`${locals.ytdlBaseUrl}/api/downloads`, {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify(body)
	});

	if (!res.ok) {
		const err = await res.json().catch(() => ({}));
		return json(err, { status: res.status });
	}

	const result = await res.json();

	locals.youtubeDownloadRepo.upsert({
		download_id: result.downloadId,
		url: body.url,
		video_id: body.videoId,
		title: body.title,
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

	return json(result, { status: 201 });
};
