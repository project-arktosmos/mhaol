import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async ({ locals }) => {
	let version: string | null = null;
	if (locals.ytdlp.isAvailable()) {
		try {
			version = await locals.ytdlp.getVersion();
		} catch {
			// ignore
		}
	}

	return json({
		available: locals.ytdlp.isAvailable(),
		version,
		downloading: false
	});
};
