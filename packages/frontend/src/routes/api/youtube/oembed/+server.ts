import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { fetchOEmbed } from 'youtube/oembed';
import { isValidYouTubeId } from 'youtube/embed';

export const GET: RequestHandler = async ({ url, locals }) => {
	const videoId = url.searchParams.get('videoId');
	if (!videoId || !isValidYouTubeId(videoId)) {
		return json({ error: 'Missing or invalid videoId parameter' }, { status: 400 });
	}

	const repo = locals.youtubeCacheRepo;

	// Check cache first
	const cached = repo.get(videoId);
	if (cached && repo.isFresh(cached.fetched_at)) {
		return json(JSON.parse(cached.data));
	}

	// Fetch from YouTube oEmbed API
	try {
		const data = await fetchOEmbed(videoId);
		repo.upsert(videoId, JSON.stringify(data));
		return json(data);
	} catch (error) {
		const message = error instanceof Error ? error.message : String(error);
		return json({ error: message }, { status: 502 });
	}
};
