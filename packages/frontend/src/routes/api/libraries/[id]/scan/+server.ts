import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { readdirSync, statSync } from 'node:fs';
import { join, extname, basename } from 'node:path';
import { MediaType } from '$types/library.type';
import type { LibraryFile, LibraryFileLink, LibraryFilesResponse } from '$types/library.type';

const VIDEO_EXTENSIONS = new Set(['.mp4', '.mkv', '.avi', '.webm', '.mov', '.m4v', '.wmv', '.flv']);
const AUDIO_EXTENSIONS = new Set(['.mp3', '.flac', '.aac', '.opus', '.ogg', '.wav', '.m4a', '.wma']);
const IMAGE_EXTENSIONS = new Set([
	'.jpg',
	'.jpeg',
	'.png',
	'.gif',
	'.bmp',
	'.webp',
	'.svg',
	'.tiff',
	'.tif',
	'.heic',
	'.heif',
	'.avif'
]);

const MEDIA_TYPE_EXTENSIONS: Record<MediaType, Set<string>> = {
	[MediaType.Video]: VIDEO_EXTENSIONS,
	[MediaType.Audio]: AUDIO_EXTENSIONS,
	[MediaType.Image]: IMAGE_EXTENSIONS
};

interface ScannedFile {
	path: string;
	extension: string;
	mediaType: MediaType;
}

function getMediaType(ext: string, allowedTypes: MediaType[]): MediaType | null {
	const lowerExt = ext.toLowerCase();
	for (const mediaType of allowedTypes) {
		if (MEDIA_TYPE_EXTENSIONS[mediaType].has(lowerExt)) {
			return mediaType;
		}
	}
	return null;
}

function scanDirectory(dir: string, allowedTypes: MediaType[]): ScannedFile[] {
	const results: ScannedFile[] = [];
	try {
		const entries = readdirSync(dir, { withFileTypes: true });
		for (const entry of entries) {
			if (entry.name.startsWith('.')) continue;
			const fullPath = join(dir, entry.name);
			if (entry.isDirectory()) {
				results.push(...scanDirectory(fullPath, allowedTypes));
			} else if (entry.isFile()) {
				const ext = extname(entry.name).toLowerCase();
				const mediaType = getMediaType(ext, allowedTypes);
				if (mediaType !== null) {
					results.push({
						path: fullPath,
						extension: ext.slice(1),
						mediaType
					});
				}
			}
		}
	} catch {
		// Directory may not exist or be inaccessible
	}
	return results;
}

export const POST: RequestHandler = async ({ params, locals }) => {
	const row = locals.libraryRepo.get(params.id);
	if (!row) {
		return json({ error: 'Library not found' }, { status: 404 });
	}

	const mediaTypes: MediaType[] = JSON.parse(row.media_types);

	try {
		statSync(row.path);
	} catch (err) {
		const code = (err as NodeJS.ErrnoException).code;
		if (code === 'ENOENT') {
			return json({ error: 'Library path does not exist' }, { status: 404 });
		}
		if (code === 'EACCES' || code === 'EPERM') {
			return json({ error: 'Permission denied' }, { status: 403 });
		}
		return json({ error: 'Cannot access library path' }, { status: 500 });
	}

	const scanned = scanDirectory(row.path, mediaTypes);

	// Sync: remove stale entries, add new files, keep existing ones untouched
	locals.libraryItemRepo.syncLibrary(
		params.id,
		scanned.map((file) => ({
			id: crypto.randomUUID(),
			library_id: params.id,
			path: file.path,
			extension: file.extension,
			media_type: file.mediaType,
			category_id: null
		}))
	);

	// Return the freshly inserted items
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
