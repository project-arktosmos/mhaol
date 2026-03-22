import { writable } from 'svelte/store';
import type {
	DisplayTMDBMovie,
	DisplayTMDBTvShow,
	DisplayTMDBMovieDetails,
	DisplayTMDBTvShowDetails
} from 'addons/tmdb/types';
import type { MediaItem } from 'frontend/types/media-card.type';
import type { LibraryItemRelated } from 'frontend/types/library-item-related.type';

export interface BrowseDetailState {
	movie: DisplayTMDBMovie | null;
	tvShow: DisplayTMDBTvShow | null;
	movieDetails: DisplayTMDBMovieDetails | null;
	tvShowDetails: DisplayTMDBTvShowDetails | null;
	libraryItem: MediaItem | null;
	relatedData: LibraryItemRelated | null;
	loading: boolean;
	fetching: boolean;
	fetched: boolean;
	downloadStatus: { state: string; progress: number } | null;
	fetchSteps: {
		terms: boolean;
		search: boolean;
		searching: boolean;
		eval: boolean;
		done: boolean;
	} | null;
}

export interface BrowseDetailCallbacks {
	onfetch?: () => void;
	ondownload?: () => void;
	onstream?: () => void;
	onp2pstream?: () => void;
	onshowsearch?: () => void;
	onclose?: () => void;
}

const initialState: BrowseDetailState = {
	movie: null,
	tvShow: null,
	movieDetails: null,
	tvShowDetails: null,
	libraryItem: null,
	relatedData: null,
	loading: false,
	fetching: false,
	fetched: false,
	downloadStatus: null,
	fetchSteps: null
};

function createBrowseDetailService() {
	const store = writable<BrowseDetailState>(initialState);
	let callbacks: BrowseDetailCallbacks = {};

	return {
		state: { subscribe: store.subscribe },
		set(state: BrowseDetailState) {
			store.set(state);
		},
		registerCallbacks(cbs: BrowseDetailCallbacks) {
			callbacks = cbs;
		},
		getCallbacks(): BrowseDetailCallbacks {
			return callbacks;
		},
		close() {
			store.set(initialState);
			callbacks = {};
		}
	};
}

export const browseDetailService = createBrowseDetailService();
