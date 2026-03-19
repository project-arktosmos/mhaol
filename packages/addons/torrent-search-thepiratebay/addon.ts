import type { PluginCompanion } from '../../../apps/video/src/lib/server/plugins/types';
import { search } from './src/search';

export const torrentSearchCompanion: PluginCompanion = {
	locals: {
		torrentSearch: () => search
	}
};
