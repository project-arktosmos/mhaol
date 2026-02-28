import type { PluginCompanion } from '../../packages/frontend/src/lib/server/plugins/types';
import { MusicBrainzCacheRepository } from './src/cache-repository';

export const musicbrainzCompanion: PluginCompanion = {
	repositories: [{ class: MusicBrainzCacheRepository, localsKey: 'musicbrainzCacheRepo' }],

	linkSources: [
		{ service: 'musicbrainz', label: 'MusicBrainz', mediaTypeId: 'audio', categoryId: null }
	]
};
