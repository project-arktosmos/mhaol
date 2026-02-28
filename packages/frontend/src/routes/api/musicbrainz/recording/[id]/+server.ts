import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { fetchRecording } from 'musicbrainz';
import { recordingToDisplay } from 'musicbrainz/transform';

export const GET: RequestHandler = async ({ params, url, locals }) => {
	const id = params.id;
	if (!id?.trim()) return json({ error: 'Invalid recording ID' }, { status: 400 });

	const refresh = url.searchParams.get('refresh') === 'true';
	const cacheRepo = locals.musicbrainzCacheRepo;

	if (!refresh) {
		const cached = cacheRepo.getRecording(id);
		if (cached && cacheRepo.isFresh(cached.fetched_at)) {
			return json(JSON.parse(cached.data));
		}
	}

	try {
		const recording = await fetchRecording(id);
		if (!recording) {
			const stale = cacheRepo.getRecording(id);
			if (stale) return json(JSON.parse(stale.data));
			return json({ error: 'Recording not found' }, { status: 404 });
		}

		const display = recordingToDisplay(recording);
		cacheRepo.upsertRecording(id, JSON.stringify(display));
		return json(display);
	} catch (err) {
		const stale = cacheRepo.getRecording(id);
		if (stale) return json(JSON.parse(stale.data));

		const message = err instanceof Error ? err.message : String(err);
		return json({ error: message }, { status: 500 });
	}
};
