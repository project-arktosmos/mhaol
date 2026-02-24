import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

let downloading = false;

export const POST: RequestHandler = async ({ locals }) => {
	if (downloading) {
		return json({ error: 'yt-dlp download already in progress' }, { status: 409 });
	}

	downloading = true;
	try {
		const path = await locals.ytdlp.downloadBinary();
		return json({ path });
	} catch (err) {
		return json({ error: err instanceof Error ? err.message : String(err) }, { status: 500 });
	} finally {
		downloading = false;
	}
};
