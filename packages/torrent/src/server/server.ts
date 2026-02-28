import { createServer, type IncomingMessage, type ServerResponse } from 'node:http';
import { join } from 'node:path';
import { getDatabase } from 'database';
import { TorrentDownloadRepository, MetadataRepository, LibraryRepository } from 'database/repositories';
import { TorrentManagerService } from './services/torrent-manager.service.js';
import { SSEBroadcasterService } from './services/sse-broadcaster.service.js';

const PORT = Number(process.env.TORRENT_PORT ?? 3050);
const DOWNLOAD_DIR = process.env.TORRENT_DOWNLOAD_DIR ?? join(process.env.HOME ?? '/tmp', 'Downloads', 'torrents');

// --- Database ---

const db = getDatabase(process.env.DB_PATH ? { dbPath: process.env.DB_PATH } : undefined);

// Ensure plugin schema exists
db.exec(`CREATE TABLE IF NOT EXISTS torrent_downloads (
    info_hash TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    size INTEGER NOT NULL DEFAULT 0,
    progress REAL NOT NULL DEFAULT 0,
    state TEXT NOT NULL DEFAULT 'initializing',
    download_speed INTEGER NOT NULL DEFAULT 0,
    upload_speed INTEGER NOT NULL DEFAULT 0,
    peers INTEGER NOT NULL DEFAULT 0,
    seeds INTEGER NOT NULL DEFAULT 0,
    added_at INTEGER NOT NULL,
    eta INTEGER,
    output_path TEXT,
    source TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TRIGGER IF NOT EXISTS torrent_downloads_updated_at
    AFTER UPDATE ON torrent_downloads
    FOR EACH ROW
BEGIN
    UPDATE torrent_downloads SET updated_at = datetime('now') WHERE info_hash = OLD.info_hash;
END;`);

const torrentDownloadRepo = new TorrentDownloadRepository(db);
const metadataRepo = new MetadataRepository(db);
const libraryRepo = new LibraryRepository(db);

// --- Services ---

const broadcaster = new SSEBroadcasterService();
const manager = new TorrentManagerService(broadcaster);

// Resolve download path
function resolveDownloadPath(): string {
	const libId = metadataRepo.getValue<string>('torrent.libraryId') as string | undefined;
	if (libId) {
		const lib = libraryRepo.get(libId);
		if (lib) return lib.path;
	}
	const allLibs = libraryRepo.getAll();
	if (allLibs.length > 0) return allLibs[0].path;
	return DOWNLOAD_DIR;
}

const downloadPath = resolveDownloadPath();
manager.initialize({ downloadPath });

manager.setPersistenceCallback((torrents) => {
	for (const t of torrents) {
		torrentDownloadRepo.upsert({
			info_hash: t.infoHash,
			name: t.name,
			size: t.size,
			progress: t.progress,
			state: t.state,
			download_speed: t.downloadSpeed,
			upload_speed: t.uploadSpeed,
			peers: t.peers,
			seeds: t.seeds,
			added_at: t.addedAt,
			eta: t.eta,
			output_path: t.outputPath,
			source: ''
		});
	}
});

console.log(`[torrent] Download directory: ${downloadPath}`);

// --- HTTP helpers ---

function jsonResponse(res: ServerResponse, data: unknown, status = 200): void {
	res.writeHead(status, { 'Content-Type': 'application/json' });
	res.end(JSON.stringify(data));
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
			return jsonResponse(res, {
				initialized: manager.isInitialized(),
				download_path: manager.getConfig().downloadPath,
				stats: manager.stats()
			});
		}

		// GET /torrents
		if (method === 'GET' && path === '/torrents') {
			return jsonResponse(res, manager.list());
		}

		// POST /torrents
		if (method === 'POST' && path === '/torrents') {
			const raw = await readBody(req);
			const body = parseJson(raw) as { source?: string; downloadPath?: string; paused?: boolean } | null;
			if (!body?.source) {
				return jsonResponse(res, { error: 'Missing required field: source' }, 400);
			}
			try {
				const info = await manager.add({
					source: body.source,
					downloadPath: body.downloadPath,
					paused: body.paused
				});
				return jsonResponse(res, info, 201);
			} catch (err) {
				const message = err instanceof Error ? err.message : String(err);
				return jsonResponse(res, { error: message }, 400);
			}
		}

		// POST /torrents/remove-all
		if (method === 'POST' && path === '/torrents/remove-all') {
			const removed = await manager.removeAll();
			return jsonResponse(res, { removed });
		}

		// GET /torrents/events (SSE)
		if (method === 'GET' && path === '/torrents/events') {
			return broadcaster.createNodeStream(req, res);
		}

		// Routes with infoHash: /torrents/:infoHash[/action]
		const torrentMatch = path.match(/^\/torrents\/([a-f0-9]+)(\/.*)?$/);
		if (torrentMatch) {
			const infoHash = torrentMatch[1];
			const action = torrentMatch[2] ?? '';

			// DELETE /torrents/:infoHash
			if (method === 'DELETE' && action === '') {
				try {
					await manager.remove(infoHash);
					return jsonResponse(res, { ok: true });
				} catch (err) {
					const message = err instanceof Error ? err.message : String(err);
					return jsonResponse(res, { error: message }, 404);
				}
			}

			// POST /torrents/:infoHash/pause
			if (method === 'POST' && action === '/pause') {
				try {
					manager.pause(infoHash);
					return jsonResponse(res, { ok: true });
				} catch (err) {
					const message = err instanceof Error ? err.message : String(err);
					return jsonResponse(res, { error: message }, 404);
				}
			}

			// POST /torrents/:infoHash/resume
			if (method === 'POST' && action === '/resume') {
				try {
					manager.resume(infoHash);
					return jsonResponse(res, { ok: true });
				} catch (err) {
					const message = err instanceof Error ? err.message : String(err);
					return jsonResponse(res, { error: message }, 404);
				}
			}
		}

		// POST /storage/clear
		if (method === 'POST' && path === '/storage/clear') {
			manager.clearStorage();
			torrentDownloadRepo.deleteAll();
			return jsonResponse(res, { ok: true });
		}

		// GET /debug
		if (method === 'GET' && path === '/debug') {
			return jsonResponse(res, { logs: manager.debugInfo() });
		}

		// GET /config
		if (method === 'GET' && path === '/config') {
			const config = manager.getConfig();
			const libraryId = (metadataRepo.getValue<string>('torrent.libraryId') ?? '') as string;
			return jsonResponse(res, { download_path: config.downloadPath, library_id: libraryId });
		}

		// PUT /config
		if (method === 'PUT' && path === '/config') {
			const raw = await readBody(req);
			const body = parseJson(raw) as { library_id?: string; download_path?: string } | null;

			if (body?.library_id !== undefined) {
				metadataRepo.set('torrent.libraryId', body.library_id);
				const lib = libraryRepo.get(body.library_id);
				if (lib) {
					manager.updateConfig({ downloadPath: lib.path });
				}
			} else if (body?.download_path) {
				manager.updateConfig({ downloadPath: body.download_path });
			}

			const config = manager.getConfig();
			return jsonResponse(res, { download_path: config.downloadPath });
		}

		// 404
		jsonResponse(res, { error: 'Not found' }, 404);
	} catch (err) {
		console.error('[torrent] Request error:', err);
		jsonResponse(res, { error: 'Internal server error' }, 500);
	}
});

// --- Lifecycle ---

server.listen(PORT, () => {
	console.log(`[torrent] Server listening on port ${PORT}`);
});

process.on('SIGINT', () => {
	manager.destroy();
	broadcaster.destroy();
	server.close();
	process.exit(0);
});

process.on('SIGTERM', () => {
	manager.destroy();
	broadcaster.destroy();
	server.close();
	process.exit(0);
});
