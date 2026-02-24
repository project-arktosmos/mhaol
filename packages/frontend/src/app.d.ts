import type { YtDlpService, DownloadManagerService, SSEBroadcasterService } from 'yt-download/services';
import type {
	TorrentManagerService,
	SSEBroadcasterService as TorrentBroadcasterService
} from 'torrent/services';
import type {
	SettingsRepository,
	MetadataRepository,
	YouTubeDownloadRepository,
	TorrentDownloadRepository
} from 'database/repositories';

declare global {
	namespace App {
		interface Locals {
			ytdlp: YtDlpService;
			downloadManager: DownloadManagerService;
			sseBroadcaster: SSEBroadcasterService;
			settingsRepo: SettingsRepository;
			metadataRepo: MetadataRepository;
			youtubeDownloadRepo: YouTubeDownloadRepository;
			torrentDownloadRepo: TorrentDownloadRepository;
			torrentManager: TorrentManagerService;
			torrentBroadcaster: TorrentBroadcasterService;
		}
	}
}

export {};
