import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { refreshPoToken, getCachedPoToken } from '$lib/server/po-token';

export const POST: RequestHandler = async ({ locals }) => {
	try {
		const { poToken, visitorData } = await refreshPoToken();

		// Sync to Rust server
		try {
			await fetch(`${locals.ytdlBaseUrl}/api/config`, {
				method: 'PUT',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ poToken, visitorData })
			});
		} catch {
			console.warn('[pot/refresh] Failed to sync to Rust server');
		}

		// Persist
		locals.settingsRepo.set('youtube.poToken', poToken);
		locals.settingsRepo.set('youtube.visitorData', visitorData);

		return json({ ok: true, visitorData: visitorData.substring(0, 16) + '...' });
	} catch (e) {
		return json({ error: `Failed to generate PO token: ${e}` }, { status: 500 });
	}
};

export const GET: RequestHandler = async () => {
	const cached = getCachedPoToken();

	if (!cached) {
		return json({ status: 'no_token', message: 'No PO token has been generated yet' });
	}

	const ageMs = Date.now() - cached.generatedAt;
	const ageMinutes = Math.floor(ageMs / 60000);

	return json({
		status: 'active',
		ageMinutes,
		visitorData: cached.visitorData.substring(0, 16) + '...'
	});
};
