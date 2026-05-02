import { ObjectServiceClass } from '$services/classes/object-service.class';

export type MovieTvViewMode = 'covers' | 'landscapes';

interface MovieTvViewSettings {
	id: string;
	mode: MovieTvViewMode;
}

const initialSettings: MovieTvViewSettings = {
	id: 'movie-tv-view-mode',
	mode: 'covers'
};

class MovieTvViewModeService extends ObjectServiceClass<MovieTvViewSettings> {
	constructor() {
		super('movie-tv-view-mode', initialSettings);
	}

	toggle(): void {
		const current = this.get();
		this.set({ ...current, mode: current.mode === 'covers' ? 'landscapes' : 'covers' });
	}

	currentMode(): MovieTvViewMode {
		return this.get().mode;
	}
}

export const movieTvViewModeService = new MovieTvViewModeService();
