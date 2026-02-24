export { getDatabase, closeDatabase, isDatabaseOpen } from './connection.js';
export { initializeSchema } from './schema.js';
export { SettingsRepository } from './repositories/settings.repository.js';
export { MetadataRepository } from './repositories/metadata.repository.js';
export { YouTubeDownloadRepository } from './repositories/youtube-download.repository.js';
export { TorrentDownloadRepository } from './repositories/torrent-download.repository.js';
export { LibraryRepository } from './repositories/library.repository.js';
export type {
	SettingRow,
	MetadataRow,
	DatabaseConfig,
	YouTubeDownloadRow,
	TorrentDownloadRow,
	LibraryRow
} from './types.js';
