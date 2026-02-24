import type {
	TorrentManagerService,
	SSEBroadcasterService as TorrentBroadcasterService
} from 'torrent/services';
import type {
	SettingsRepository,
	MetadataRepository,
	YouTubeDownloadRepository,
	TorrentDownloadRepository,
	LibraryRepository
} from 'database/repositories';

declare global {
	namespace App {
		interface Locals {
			ytdlBaseUrl: string;
			ytdlAvailable: boolean;
			settingsRepo: SettingsRepository;
			metadataRepo: MetadataRepository;
			youtubeDownloadRepo: YouTubeDownloadRepository;
			torrentDownloadRepo: TorrentDownloadRepository;
			torrentManager: TorrentManagerService;
			torrentBroadcaster: TorrentBroadcasterService;
			libraryRepo: LibraryRepository;
			streamServerAvailable: boolean;
			ytdlOutputDir: string;
		}
	}
}

export {};
