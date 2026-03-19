import type { PluginCompanion } from '../../packages/mhaol-video/src/lib/server/plugins/types';
import { search } from './src/search';

export const torrentSearchCompanion: PluginCompanion = {
	locals: {
		torrentSearch: () => search
	}
};
