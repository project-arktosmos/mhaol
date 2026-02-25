import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async ({ locals }) => {
	const config = locals.torrentManager.getConfig();
	const libraryId = (locals.metadataRepo.getValue<string>('torrent.libraryId') ?? '') as string;
	return json({ download_path: config.downloadPath, library_id: libraryId });
};

export const PUT: RequestHandler = async ({ request, locals }) => {
	const body = await request.json();

	if (body.library_id !== undefined) {
		locals.metadataRepo.set('torrent.libraryId', body.library_id as string);
		const lib = locals.libraryRepo.get(body.library_id as string);
		if (lib) {
			locals.torrentManager.updateConfig({ downloadPath: lib.path });
		}
	} else if (body.download_path) {
		locals.torrentManager.updateConfig({ downloadPath: body.download_path });
	}

	const config = locals.torrentManager.getConfig();
	return json({ download_path: config.downloadPath });
};
