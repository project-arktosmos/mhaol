import type { PluginCompanion } from '../types';
import { TorrentDownloadRepository } from 'database/repositories';

export const torrentCompanion: PluginCompanion = {
	repositories: [{ class: TorrentDownloadRepository, localsKey: 'torrentDownloadRepo' }]
};
