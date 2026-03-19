import type { PluginCompanion } from '../../../apps/video/src/lib/server/plugins/types';
import { TmdbCacheRepository } from './src/cache-repository';

export const tmdbCompanion: PluginCompanion = {
	repositories: [{ class: TmdbCacheRepository, localsKey: 'tmdbCacheRepo' }],

	linkSources: [
		{ service: 'tmdb', label: 'TMDB', mediaTypeId: 'video', categoryId: 'movies' },
		{ service: 'tmdb', label: 'TMDB', mediaTypeId: 'video', categoryId: 'tv' }
	],

	locals: {
		tmdbApiKey: (ctx) => () => ctx.settingsRepo.get('tmdb.apiKey') ?? ''
	}
};
