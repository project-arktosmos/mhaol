import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { readdirSync, statSync } from 'node:fs';
import { join, extname, basename } from 'node:path';
import { MediaType } from '$types/library.type';
import type { LibraryFile, LibraryFilesResponse } from '$types/library.type';

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
	[MediaType.Music]: AUDIO_EXTENSIONS,
	[MediaType.Images]: IMAGE_EXTENSIONS
};

function getMediaType(ext: string, allowedTypes: MediaType[]): MediaType | null {
	const lowerExt = ext.toLowerCase();
	for (const mediaType of allowedTypes) {
		if (MEDIA_TYPE_EXTENSIONS[mediaType].has(lowerExt)) {
			return mediaType;
		}
	}
	return null;
}

function findFiles(dir: string, allowedTypes: MediaType[]): LibraryFile[] {
	const results: LibraryFile[] = [];
	try {
		const entries = readdirSync(dir, { withFileTypes: true });
		for (const entry of entries) {
			if (entry.name.startsWith('.')) continue;
			const fullPath = join(dir, entry.name);
			if (entry.isDirectory()) {
				results.push(...findFiles(fullPath, allowedTypes));
			} else if (entry.isFile()) {
				const ext = extname(entry.name).toLowerCase();
				const mediaType = getMediaType(ext, allowedTypes);
				if (mediaType !== null) {
					let size = 0;
					try {
						size = statSync(fullPath).size;
					} catch {
						// Skip if stat fails
					}
					results.push({
						name: basename(entry.name),
						path: fullPath,
						size,
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

export const GET: RequestHandler = async ({ params, locals }) => {
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

	const files = findFiles(row.path, mediaTypes);
	files.sort((a, b) => a.name.localeCompare(b.name));

	const response: LibraryFilesResponse = {
		libraryId: row.id,
		libraryPath: row.path,
		files
	};

	return json(response);
};
