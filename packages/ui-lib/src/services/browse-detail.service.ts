import { writable } from 'svelte/store';
import type { BrowseDetailState, BrowseDetailCallbacks } from 'ui-lib/types/browse-detail.type';

export type { BrowseDetailState, BrowseDetailCallbacks };
export type { BrowseDetailDomain, PhotoImageData } from 'ui-lib/types/browse-detail.type';

const initialState: BrowseDetailState = {
	domain: null,

	// Movie/TV
	movie: null,
	tvShow: null,
	movieDetails: null,
	tvShowDetails: null,
	tvSeasonDetails: [],
	libraryItem: null,
	relatedData: null,
	loading: false,
	fetching: false,
	fetched: false,
	downloadStatus: null,
	fetchSteps: null,

	// Music
	musicAlbum: null,
	musicRelease: null,
	musicTorrent: null,

	// Photo
	photoImage: null,
	photoTags: [],
	photoTagging: false,

	// Videogame
	videogame: null,
	videogameDetails: null,
	videogameDetailsLoading: false,

	// YouTube
	youtubeVideo: null,

	// Book
	book: null,
	bookDetails: null
};

function createBrowseDetailService() {
	const store = writable<BrowseDetailState>(initialState);
	let callbacks: BrowseDetailCallbacks = {};

	return {
		state: { subscribe: store.subscribe },
		set(state: Partial<BrowseDetailState>) {
			store.update((current) => ({ ...current, ...state }));
		},
		update(fn: (state: BrowseDetailState) => Partial<BrowseDetailState>) {
			store.update((current) => ({ ...current, ...fn(current) }));
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
