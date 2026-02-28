import { createServer, type IncomingMessage, type ServerResponse } from 'node:http';
import { readFile } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import { basename } from 'node:path';
import { getDatabase } from 'database';
import { ImageTagRepository, LibraryItemRepository, LibraryRepository } from 'database/repositories';
import { tagImage, isTaggerReady, getTaggerProgress } from './image-tagger.service.js';

const PORT = Number(process.env.IMAGE_TAGGER_PORT ?? 3060);

// --- Database ---

const db = getDatabase(process.env.DB_PATH ? { dbPath: process.env.DB_PATH } : undefined);

// Ensure plugin schema exists
db.exec(`CREATE TABLE IF NOT EXISTS image_tags (
    id TEXT PRIMARY KEY,
    library_item_id TEXT NOT NULL REFERENCES library_items(id) ON DELETE CASCADE,
    tag TEXT NOT NULL,
    score REAL NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_image_tags_library_item_id ON image_tags(library_item_id);
CREATE INDEX IF NOT EXISTS idx_image_tags_tag ON image_tags(tag);`);

const imageTagRepo = new ImageTagRepository(db);
const libraryItemRepo = new LibraryItemRepository(db);
const libraryRepo = new LibraryRepository(db);

// --- MIME types ---

const MIME_TYPES: Record<string, string> = {
	jpg: 'image/jpeg',
	jpeg: 'image/jpeg',
	png: 'image/png',
	gif: 'image/gif',
	bmp: 'image/bmp',
	webp: 'image/webp',
	svg: 'image/svg+xml',
	tiff: 'image/tiff',
	tif: 'image/tiff',
	heic: 'image/heic',
	heif: 'image/heif',
	avif: 'image/avif'
};

// --- HTTP helpers ---

function jsonResponse(res: ServerResponse, data: unknown, status = 200): void {
	res.writeHead(status, { 'Content-Type': 'application/json' });
	res.end(JSON.stringify(data));
}

function errorResponse(res: ServerResponse, message: string, status: number): void {
	jsonResponse(res, { error: message }, status);
}

function readBody(req: IncomingMessage): Promise<string> {
	return new Promise((resolve, reject) => {
		let body = '';
		req.on('data', (chunk) => (body += chunk));
		req.on('end', () => resolve(body));
		req.on('error', reject);
	});
}

function parseJson(raw: string): unknown {
	try {
		return JSON.parse(raw);
	} catch {
		return null;
	}
}

// --- Routing ---

const server = createServer(async (req, res) => {
	const url = new URL(req.url ?? '/', `http://localhost:${PORT}`);
	const path = url.pathname;
	const method = req.method ?? 'GET';

	try {
		// GET /health
		if (method === 'GET' && path === '/health') {
			return jsonResponse(res, { ok: true });
		}

		// GET /status
		if (method === 'GET' && path === '/status') {
			const progress = getTaggerProgress();
			return jsonResponse(res, {
				ready: isTaggerReady(),
				status: progress.status,
				overallProgress: progress.overallProgress,
				error: progress.error
			});
		}

		// GET /images
		if (method === 'GET' && path === '/images') {
			const items = libraryItemRepo.getByMediaType('image');
			const libraries = libraryRepo.getAll();
			const libraryMap = new Map(libraries.map((lib) => [lib.id, lib.name]));

			const itemIds = items.map((item) => item.id);
			const tagsMap = imageTagRepo.getByItems(itemIds);

			const images = items.map((item) => ({
				id: item.id,
				libraryId: item.library_id,
				libraryName: libraryMap.get(item.library_id) ?? 'Unknown',
				name: basename(item.path),
				path: item.path,
				extension: item.extension,
				tags: (tagsMap[item.id] ?? []).map((t) => ({ tag: t.tag, score: t.score }))
			}));

			return jsonResponse(res, { images });
		}

		// GET /images/serve?path=...
		if (method === 'GET' && path === '/images/serve') {
			const filePath = url.searchParams.get('path');
			if (!filePath) {
				return errorResponse(res, 'Missing path parameter', 400);
			}

			const itemId = libraryItemRepo.existsByPath(filePath);
			if (!itemId) {
				return errorResponse(res, 'Path not found in library', 403);
			}

			if (!existsSync(filePath)) {
				return errorResponse(res, 'File not found on disk', 404);
			}

			const ext = filePath.split('.').pop()?.toLowerCase() ?? '';
			const mimeType = MIME_TYPES[ext] ?? 'application/octet-stream';

			const fileBuffer = await readFile(filePath);

			res.writeHead(200, {
				'Content-Type': mimeType,
				'Cache-Control': 'public, max-age=3600'
			});
			res.end(fileBuffer);
			return;
		}

		// POST /images/tag
		if (method === 'POST' && path === '/images/tag') {
			const raw = await readBody(req);
			const body = parseJson(raw) as { libraryItemId?: string; threshold?: number } | null;

			if (!body?.libraryItemId) {
				return errorResponse(res, 'Missing libraryItemId', 400);
			}

			const item = libraryItemRepo.get(body.libraryItemId);
			if (!item) {
				return errorResponse(res, 'Library item not found', 404);
			}

			const tags = await tagImage(item.path, undefined, body.threshold);

			const tagRows = tags.map((t) => ({
				id: crypto.randomUUID(),
				library_item_id: body.libraryItemId!,
				tag: t.tag,
				score: t.score
			}));

			imageTagRepo.replaceForItem(body.libraryItemId, tagRows);

			return jsonResponse(res, { libraryItemId: body.libraryItemId, tags });
		}

		// POST /images/tag-batch
		if (method === 'POST' && path === '/images/tag-batch') {
			const raw = await readBody(req);
			const body = parseJson(raw) as { libraryItemIds?: string[]; threshold?: number } | null;

			if (!body?.libraryItemIds || !Array.isArray(body.libraryItemIds) || body.libraryItemIds.length === 0) {
				return errorResponse(res, 'Missing or empty libraryItemIds array', 400);
			}

			const results: Record<string, { tag: string; score: number }[]> = {};

			for (const itemId of body.libraryItemIds) {
				const item = libraryItemRepo.get(itemId);
				if (!item) {
					console.warn(`[image-tagger] Item ${itemId} not found, skipping`);
					results[itemId] = [];
					continue;
				}

				try {
					const tags = await tagImage(item.path, undefined, body.threshold);

					const tagRows = tags.map((t) => ({
						id: crypto.randomUUID(),
						library_item_id: itemId,
						tag: t.tag,
						score: t.score
					}));

					imageTagRepo.replaceForItem(itemId, tagRows);
					results[itemId] = tags;
				} catch (e) {
					console.error(`[image-tagger] Failed to tag item ${itemId}:`, e);
					results[itemId] = [];
				}
			}

			return jsonResponse(res, { results });
		}

		// POST /images/tags (add manual tag)
		if (method === 'POST' && path === '/images/tags') {
			const raw = await readBody(req);
			const body = parseJson(raw) as { libraryItemId?: string; tag?: string } | null;

			if (!body?.libraryItemId || !body.tag?.trim()) {
				return errorResponse(res, 'Missing libraryItemId or tag', 400);
			}

			const item = libraryItemRepo.get(body.libraryItemId);
			if (!item) {
				return errorResponse(res, 'Library item not found', 404);
			}

			const trimmed = body.tag.trim().toLowerCase();
			imageTagRepo.addTag(body.libraryItemId, trimmed, 1.0);

			return jsonResponse(res, { libraryItemId: body.libraryItemId, tag: trimmed, score: 1.0 });
		}

		// DELETE /images/tags
		if (method === 'DELETE' && path === '/images/tags') {
			const raw = await readBody(req);
			const body = parseJson(raw) as { libraryItemId?: string; tag?: string } | null;

			if (!body?.libraryItemId || !body.tag) {
				return errorResponse(res, 'Missing libraryItemId or tag', 400);
			}

			imageTagRepo.deleteTag(body.libraryItemId, body.tag);

			return jsonResponse(res, { ok: true });
		}

		// 404
		jsonResponse(res, { error: 'Not found' }, 404);
	} catch (err) {
		console.error('[image-tagger] Request error:', err);
		jsonResponse(res, { error: 'Internal server error' }, 500);
	}
});

// --- Lifecycle ---

server.listen(PORT, () => {
	console.log(`[image-tagger] Server listening on port ${PORT}`);
});

process.on('SIGINT', () => {
	server.close();
	process.exit(0);
});

process.on('SIGTERM', () => {
	server.close();
	process.exit(0);
});
