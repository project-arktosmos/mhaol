import type { Handle } from '@sveltejs/kit';
import { join, dirname } from 'node:path';
import { existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { spawn, type ChildProcess } from 'node:child_process';
import {
	TorrentManagerService,
	SSEBroadcasterService as TorrentBroadcasterService
} from 'torrent/services';
import { getDatabase } from 'database';
import {
	SettingsRepository,
	MetadataRepository,
	YouTubeDownloadRepository,
	TorrentDownloadRepository,
	LibraryRepository
} from 'database/repositories';
import { getPoToken } from '$lib/server/po-token';

const __dirname = dirname(fileURLToPath(import.meta.url));
const PACKAGE_ROOT = join(__dirname, '..');
const OUTPUT_DIR =
	process.env.YTDL_OUTPUT_DIR ?? join(process.env.HOME ?? '/tmp', 'Downloads', 'youtube');
const TORRENT_DIR =
	process.env.TORRENT_DOWNLOAD_DIR ?? join(process.env.HOME ?? '/tmp', 'Downloads', 'torrents');

// Initialize database singleton and repositories
const dbPath = process.env.DB_PATH ?? undefined;
const db = getDatabase(dbPath ? { dbPath } : undefined);
const settingsRepo = new SettingsRepository(db);
const metadataRepo = new MetadataRepository(db);
const youtubeDownloadRepo = new YouTubeDownloadRepository(db);
const torrentDownloadRepo = new TorrentDownloadRepository(db);
const libraryRepo = new LibraryRepository(db);

// Seed a default Downloads library if none exist
if (libraryRepo.getAll().length === 0) {
	const defaultDownloadsPath = join(process.env.HOME ?? '/tmp', 'Downloads');
	libraryRepo.insert({
		id: crypto.randomUUID(),
		name: 'Downloads',
		path: defaultDownloadsPath,
		media_types: JSON.stringify(['video', 'images', 'music']),
		date_added: Date.now()
	});
	console.log(`[database] Created default library at ${defaultDownloadsPath}`);
}

console.log(`[database] Initialized`);

// Initialize Rust YouTube download server
const YTDL_PORT = process.env.YTDL_PORT ?? '3040';
const YTDL_BIN =
	process.env.YTDL_BIN ??
	join(PACKAGE_ROOT, '..', 'rust-yt-dlp', 'target', 'debug', 'mhaol-yt-dlp-server');

let ytdlServerProcess: ChildProcess | null = null;
let ytdlServerAvailable = false;
const ytdlBaseUrl = `http://localhost:${YTDL_PORT}`;

// Read persisted auth config to pass to Rust server on startup
const persistedPoToken = settingsRepo.get('youtube.poToken') ?? undefined;
const persistedCookies = settingsRepo.get('youtube.cookies') ?? undefined;

// Resolve YouTube output path from library
function resolveLibraryPath(metaKey: string, fallback: string): string {
	const libId = metadataRepo.getValue<string>(metaKey) as string | undefined;
	if (libId) {
		const lib = libraryRepo.get(libId);
		if (lib) return lib.path;
	}
	// Fallback: use first library if available
	const allLibs = libraryRepo.getAll();
	if (allLibs.length > 0) return allLibs[0].path;
	return fallback;
}

const persistedOutputPath = resolveLibraryPath('youtube.libraryId', OUTPUT_DIR);

if (existsSync(YTDL_BIN)) {
	ytdlServerProcess = spawn(YTDL_BIN, [], {
		env: {
			...process.env,
			YTDL_PORT,
			YTDL_OUTPUT_DIR: persistedOutputPath,
			YTDL_CORS_ORIGIN: 'http://localhost:1530',
			RUST_LOG: process.env.RUST_LOG ?? 'info',
			...(persistedPoToken ? { YTDL_PO_TOKEN: persistedPoToken } : {}),
			...(persistedCookies ? { YTDL_COOKIES: persistedCookies } : {})
		},
		stdio: ['ignore', 'pipe', 'pipe']
	});

	ytdlServerProcess.stdout?.on('data', (data: Buffer) => {
		for (const line of data.toString().trimEnd().split('\n')) {
			console.log(`[ytdl-rust] ${line}`);
		}
	});

	ytdlServerProcess.stderr?.on('data', (data: Buffer) => {
		for (const line of data.toString().trimEnd().split('\n')) {
			console.error(`[ytdl-rust] ${line}`);
		}
	});

	ytdlServerProcess.on('error', (err) => {
		console.error(`[ytdl-rust] Failed to start: ${err.message}`);
		ytdlServerAvailable = false;
	});

	ytdlServerProcess.on('exit', (code) => {
		console.log(`[ytdl-rust] Process exited with code ${code}`);
		ytdlServerAvailable = false;
		ytdlServerProcess = null;
	});

	ytdlServerAvailable = true;
	console.log(`[ytdl-rust] Started on port ${YTDL_PORT} (pid: ${ytdlServerProcess.pid})`);

	// Auto-generate and sync PO token to Rust server (only if no manual token is set)
	if (!persistedPoToken) {
		const syncPoToken = async () => {
			// Wait for Rust server to be ready
			for (let i = 0; i < 20; i++) {
				try {
					const res = await fetch(`${ytdlBaseUrl}/api/status`);
					if (res.ok) break;
				} catch {
					// Server not ready yet
				}
				await new Promise((r) => setTimeout(r, 500));
			}

			try {
				const { poToken, visitorData } = await getPoToken();
				await fetch(`${ytdlBaseUrl}/api/config`, {
					method: 'PUT',
					headers: { 'Content-Type': 'application/json' },
					body: JSON.stringify({ poToken, visitorData })
				});
				settingsRepo.set('youtube.poToken', poToken);
				settingsRepo.set('youtube.visitorData', visitorData);
				console.log('[po-token] Auto-generated and synced to Rust server');
			} catch (e) {
				console.warn(`[po-token] Failed to auto-generate: ${e}`);
			}
		};

		syncPoToken();

		// Refresh PO token every 6 hours
		setInterval(
			async () => {
				try {
					const { refreshPoToken } = await import('$lib/server/po-token');
					const { poToken, visitorData } = await refreshPoToken();
					await fetch(`${ytdlBaseUrl}/api/config`, {
						method: 'PUT',
						headers: { 'Content-Type': 'application/json' },
						body: JSON.stringify({ poToken, visitorData })
					});
					settingsRepo.set('youtube.poToken', poToken);
					settingsRepo.set('youtube.visitorData', visitorData);
					console.log('[po-token] Refreshed and synced to Rust server');
				} catch (e) {
					console.warn(`[po-token] Failed to refresh: ${e}`);
				}
			},
			6 * 60 * 60 * 1000
		);
	}
} else {
	console.warn(`[ytdl-rust] Binary not found at ${YTDL_BIN}, YouTube downloads disabled`);
	console.warn(`[ytdl-rust] Run 'pnpm ytdl-rust:build' to build it`);
}

// Initialize torrent services
const torrentBroadcaster = new TorrentBroadcasterService();
const torrentManager = new TorrentManagerService(torrentBroadcaster);
const torrentDownloadPath = resolveLibraryPath('torrent.libraryId', TORRENT_DIR);
torrentManager.initialize({ downloadPath: torrentDownloadPath });

// Wire torrent persistence
torrentManager.setPersistenceCallback((torrents) => {
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

console.log(`[torrent] Download directory: ${TORRENT_DIR}`);

// Initialize p2p-stream server
const P2P_STREAM_PORT = process.env.P2P_STREAM_PORT ?? '3001';
const P2P_STREAM_BIN =
	process.env.P2P_STREAM_BIN ??
	join(PACKAGE_ROOT, '..', 'p2p-stream-server', 'target', 'debug', 'p2p-stream-server');

let streamServerProcess: ChildProcess | null = null;
let streamServerAvailable = false;

if (existsSync(P2P_STREAM_BIN)) {
	streamServerProcess = spawn(P2P_STREAM_BIN, [], {
		env: {
			...process.env,
			P2P_STREAM_PORT,
			P2P_STREAM_ALLOWED_ORIGINS: 'http://localhost:1530',
			RUST_LOG: process.env.RUST_LOG ?? 'info'
		},
		stdio: ['ignore', 'pipe', 'pipe']
	});

	streamServerProcess.stdout?.on('data', (data: Buffer) => {
		for (const line of data.toString().trimEnd().split('\n')) {
			console.log(`[p2p-stream] ${line}`);
		}
	});

	streamServerProcess.stderr?.on('data', (data: Buffer) => {
		for (const line of data.toString().trimEnd().split('\n')) {
			console.error(`[p2p-stream] ${line}`);
		}
	});

	streamServerProcess.on('error', (err) => {
		console.error(`[p2p-stream] Failed to start: ${err.message}`);
		streamServerAvailable = false;
	});

	streamServerProcess.on('exit', (code) => {
		console.log(`[p2p-stream] Process exited with code ${code}`);
		streamServerAvailable = false;
		streamServerProcess = null;
	});

	streamServerAvailable = true;
	console.log(`[p2p-stream] Started on port ${P2P_STREAM_PORT} (pid: ${streamServerProcess.pid})`);
} else {
	console.warn(`[p2p-stream] Binary not found at ${P2P_STREAM_BIN}, streaming disabled`);
	console.warn(`[p2p-stream] Run 'pnpm p2p-server:build' to build it`);
}

// Cleanup on process exit
function cleanupChildProcesses() {
	if (ytdlServerProcess) {
		ytdlServerProcess.kill();
		ytdlServerProcess = null;
	}
	if (streamServerProcess) {
		streamServerProcess.kill();
		streamServerProcess = null;
	}
}
process.on('exit', cleanupChildProcesses);
process.on('SIGINT', () => {
	cleanupChildProcesses();
	process.exit(0);
});
process.on('SIGTERM', () => {
	cleanupChildProcesses();
	process.exit(0);
});

export const handle: Handle = async ({ event, resolve }) => {
	event.locals.ytdlBaseUrl = ytdlBaseUrl;
	event.locals.ytdlAvailable = ytdlServerAvailable;
	event.locals.settingsRepo = settingsRepo;
	event.locals.metadataRepo = metadataRepo;
	event.locals.youtubeDownloadRepo = youtubeDownloadRepo;
	event.locals.torrentDownloadRepo = torrentDownloadRepo;
	event.locals.torrentManager = torrentManager;
	event.locals.torrentBroadcaster = torrentBroadcaster;
	event.locals.libraryRepo = libraryRepo;
	event.locals.streamServerAvailable = streamServerAvailable;

	return resolve(event);
};
