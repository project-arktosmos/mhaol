import type { Handle } from '@sveltejs/kit';
import { join, dirname } from 'node:path';
import { existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { spawn, type ChildProcess } from 'node:child_process';
import { YtDlpService, DownloadManagerService, SSEBroadcasterService } from 'yt-download/services';
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

const __dirname = dirname(fileURLToPath(import.meta.url));
const PACKAGE_ROOT = join(__dirname, '..');
const OUTPUT_DIR = process.env.YTDL_OUTPUT_DIR ?? join(PACKAGE_ROOT, 'downloads');
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

console.log(`[database] Initialized`);

// Initialize YouTube download services
const ytdlp = new YtDlpService();
ytdlp.initialize(OUTPUT_DIR);

const broadcaster = new SSEBroadcasterService();
const downloadManager = new DownloadManagerService(ytdlp, broadcaster);
downloadManager.initialize(OUTPUT_DIR);

// Wire YouTube persistence
downloadManager.setPersistenceCallback((progress) => {
	youtubeDownloadRepo.upsert({
		download_id: progress.downloadId,
		url: progress.url,
		video_id: progress.videoId,
		title: progress.title,
		state: progress.state,
		progress: progress.progress,
		downloaded_bytes: progress.downloadedBytes,
		total_bytes: progress.totalBytes,
		output_path: progress.outputPath,
		error: progress.error,
		mode: progress.mode,
		quality: progress.quality,
		format: progress.format,
		video_quality: progress.videoQuality,
		video_format: progress.videoFormat,
		thumbnail_url: progress.thumbnailUrl,
		duration_seconds: progress.durationSeconds
	});
});

downloadManager.setDeleteCallback((downloadIds) => {
	for (const id of downloadIds) {
		youtubeDownloadRepo.delete(id);
	}
});

console.log(`[ytdl] Output directory: ${OUTPUT_DIR}`);
console.log(`[ytdl] yt-dlp available: ${ytdlp.isAvailable()}`);

// Initialize torrent services
const torrentBroadcaster = new TorrentBroadcasterService();
const torrentManager = new TorrentManagerService(torrentBroadcaster);
torrentManager.initialize({ downloadPath: TORRENT_DIR });

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
function cleanupStreamServer() {
	if (streamServerProcess) {
		streamServerProcess.kill();
		streamServerProcess = null;
	}
}
process.on('exit', cleanupStreamServer);
process.on('SIGINT', () => {
	cleanupStreamServer();
	process.exit(0);
});
process.on('SIGTERM', () => {
	cleanupStreamServer();
	process.exit(0);
});

export const handle: Handle = async ({ event, resolve }) => {
	event.locals.ytdlp = ytdlp;
	event.locals.downloadManager = downloadManager;
	event.locals.sseBroadcaster = broadcaster;
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
