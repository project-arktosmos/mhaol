import type { PluginCompanion } from '../types';
import { search } from 'torrent-search';

export const torrentSearchCompanion: PluginCompanion = {
	locals: {
		torrentSearch: () => search
	}
};
