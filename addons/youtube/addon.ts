import type { PluginCompanion } from '../../packages/frontend/src/lib/server/plugins/types';
import { YouTubeCacheRepository } from './src/cache-repository';

export const youtubeCompanion: PluginCompanion = {
	repositories: [{ class: YouTubeCacheRepository, localsKey: 'youtubeCacheRepo' }],

	linkSources: [
		{ service: 'youtube', label: 'YouTube', mediaTypeId: 'video', categoryId: null },
		{ service: 'youtube', label: 'YouTube', mediaTypeId: 'audio', categoryId: null }
	]
};
