import type { Handle } from '@sveltejs/kit';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
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
	TorrentDownloadRepository
} from 'database/repositories';

const __dirname = dirname(fileURLToPath(import.meta.url));
const PACKAGE_ROOT = join(__dirname, '..');
const OUTPUT_DIR = process.env.YTDL_OUTPUT_DIR ?? join(PACKAGE_ROOT, 'downloads');
const TORRENT_DIR =
	process.env.TORRENT_DOWNLOAD_DIR ??
	join(process.env.HOME ?? '/tmp', 'Downloads', 'torrents');

// Initialize database singleton and repositories
const dbPath = process.env.DB_PATH ?? undefined;
const db = getDatabase(dbPath ? { dbPath } : undefined);
const settingsRepo = new SettingsRepository(db);
const metadataRepo = new MetadataRepository(db);
const youtubeDownloadRepo = new YouTubeDownloadRepository(db);
const torrentDownloadRepo = new TorrentDownloadRepository(db);

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

	return resolve(event);
};
