import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { basename } from 'node:path';
import type { LibraryFile, LibraryFileLink, LibraryFilesResponse } from '$types/library.type';
import type { MediaType } from '$types/library.type';

export const GET: RequestHandler = async ({ params, locals }) => {
	const row = locals.libraryRepo.get(params.id);
	if (!row) {
		return json({ error: 'Library not found' }, { status: 404 });
	}

	const items = locals.libraryItemRepo.getByLibrary(params.id);

	const files: LibraryFile[] = items.map((item) => {
		const linkRows = locals.libraryItemLinkRepo.getByItem(item.id);
		const links: Record<string, LibraryFileLink> = {};
		for (const link of linkRows) {
			links[link.service] = {
				serviceId: link.service_id,
				seasonNumber: link.season_number,
				episodeNumber: link.episode_number
			};
		}

		return {
			id: item.id,
			name: basename(item.path),
			path: item.path,
			extension: item.extension,
			mediaType: item.media_type as MediaType,
			categoryId: item.category_id,
			links
		};
	});

	const response: LibraryFilesResponse = {
		libraryId: row.id,
		libraryPath: row.path,
		files
	};

	return json(response);
};
