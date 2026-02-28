import type {
	SettingsRepository,
	MetadataRepository,
	YouTubeDownloadRepository,
	TorrentDownloadRepository,
	LibraryRepository,
	LibraryItemRepository,
	LibraryItemLinkRepository,
	LinkSourceRepository,
	MediaTypeRepository,
	CategoryRepository
} from 'database/repositories';
import type { TmdbCacheRepository } from 'tmdb/cache-repository';
import type { PluginConnector } from '$lib/server/plugins/connector';
import type { search } from 'torrent-search';

declare global {
	namespace App {
		interface Locals {
			pluginConnector: PluginConnector;
			ytdlBaseUrl: string;
			ytdlAvailable: boolean;
			settingsRepo: SettingsRepository;
			metadataRepo: MetadataRepository;
			youtubeDownloadRepo: YouTubeDownloadRepository;
			torrentDownloadRepo: TorrentDownloadRepository;
			torrentBaseUrl: string;
			torrentAvailable: boolean;
			libraryRepo: LibraryRepository;
			libraryItemRepo: LibraryItemRepository;
			libraryItemLinkRepo: LibraryItemLinkRepository;
			mediaTypeRepo: MediaTypeRepository;
			categoryRepo: CategoryRepository;
			linkSourceRepo: LinkSourceRepository;
			imageTaggerBaseUrl: string;
			imageTaggerAvailable: boolean;
			streamServerAvailable: boolean;
			signalingDevUrl: string;
			signalingDevAvailable: boolean;
			signalingPartyUrl: string;
			torrentSearch: typeof search;
			tmdbCacheRepo: TmdbCacheRepository;
			tmdbApiKey: () => string;
		}
	}
}

export {};
