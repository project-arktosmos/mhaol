import { describe, it, expect, vi, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import { browseDetailService } from '../../src/services/browse-detail.service';
import type {
	BrowseDetailState,
	BrowseDetailCallbacks
} from '../../src/services/browse-detail.service';

const initialState: BrowseDetailState = {
	domain: null,
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
	musicAlbum: null,
	musicRelease: null,
	musicTorrent: null,
	photoImage: null,
	photoTags: [],
	photoTagging: false,
	videogame: null,
	videogameDetails: null,
	videogameDetailsLoading: false,
	youtubeVideo: null,
	book: null,
	bookDetails: null
};

describe('BrowseDetailService', () => {
	beforeEach(() => {
		browseDetailService.close();
	});

	// ===== Initial state =====

	it('has correct initial state', () => {
		const state = get(browseDetailService.state);
		expect(state.domain).toBeNull();
		expect(state.loading).toBe(false);
		expect(state.fetching).toBe(false);
		expect(state.fetched).toBe(false);
		expect(state.movie).toBeNull();
		expect(state.tvShow).toBeNull();
		expect(state.book).toBeNull();
		expect(state.videogame).toBeNull();
		expect(state.youtubeVideo).toBeNull();
		expect(state.photoImage).toBeNull();
		expect(state.musicAlbum).toBeNull();
	});

	// ===== set =====

	it('set merges partial state into current state', () => {
		browseDetailService.set({ domain: 'movie', loading: true });

		const state = get(browseDetailService.state);
		expect(state.domain).toBe('movie');
		expect(state.loading).toBe(true);
		expect(state.fetching).toBe(false);
	});

	it('set overwrites only specified fields', () => {
		browseDetailService.set({ domain: 'tv', fetching: true });
		browseDetailService.set({ loading: true });

		const state = get(browseDetailService.state);
		expect(state.domain).toBe('tv');
		expect(state.fetching).toBe(true);
		expect(state.loading).toBe(true);
	});

	it('set can update movie domain fields', () => {
		const fakeMovie = { id: 1, title: 'Test Movie' } as never;
		browseDetailService.set({ domain: 'movie', movie: fakeMovie });

		const state = get(browseDetailService.state);
		expect(state.domain).toBe('movie');
		expect(state.movie).toEqual({ id: 1, title: 'Test Movie' });
	});

	it('set can update book domain fields', () => {
		const fakeBook = { key: 'OL123', title: 'Test Book' } as never;
		browseDetailService.set({ domain: 'book', book: fakeBook });

		const state = get(browseDetailService.state);
		expect(state.domain).toBe('book');
		expect(state.book).toEqual({ key: 'OL123', title: 'Test Book' });
	});

	it('set can update videogame domain fields', () => {
		browseDetailService.set({ domain: 'videogame', videogameDetailsLoading: true });

		const state = get(browseDetailService.state);
		expect(state.domain).toBe('videogame');
		expect(state.videogameDetailsLoading).toBe(true);
	});

	it('set can update photo domain fields', () => {
		browseDetailService.set({ domain: 'photo', photoTagging: true, photoTags: ['nature'] });

		const state = get(browseDetailService.state);
		expect(state.domain).toBe('photo');
		expect(state.photoTagging).toBe(true);
		expect(state.photoTags).toEqual(['nature']);
	});

	it('set can update download status', () => {
		browseDetailService.set({ downloadStatus: { state: 'downloading', progress: 42 } });

		const state = get(browseDetailService.state);
		expect(state.downloadStatus).toEqual({ state: 'downloading', progress: 42 });
	});

	it('set can update fetch steps', () => {
		const steps = { terms: true, search: true, searching: false, eval: false, done: false };
		browseDetailService.set({ fetchSteps: steps });

		const state = get(browseDetailService.state);
		expect(state.fetchSteps).toEqual(steps);
	});

	// ===== update =====

	it('update applies a function to current state', () => {
		browseDetailService.set({ domain: 'movie', loading: false });
		browseDetailService.update((s) => ({ loading: !s.loading }));

		const state = get(browseDetailService.state);
		expect(state.loading).toBe(true);
		expect(state.domain).toBe('movie');
	});

	it('update can toggle fetching state', () => {
		browseDetailService.set({ fetching: false });
		browseDetailService.update(() => ({ fetching: true, fetched: false }));

		const state = get(browseDetailService.state);
		expect(state.fetching).toBe(true);
		expect(state.fetched).toBe(false);
	});

	it('update receives the latest state', () => {
		browseDetailService.set({ domain: 'tv', loading: true });
		browseDetailService.update((s) => {
			expect(s.domain).toBe('tv');
			expect(s.loading).toBe(true);
			return { loading: false };
		});

		const state = get(browseDetailService.state);
		expect(state.loading).toBe(false);
	});

	// ===== close =====

	it('close resets state to initial values', () => {
		browseDetailService.set({
			domain: 'movie',
			loading: true,
			fetching: true,
			fetched: true,
			downloadStatus: { state: 'done', progress: 100 }
		});

		browseDetailService.close();

		const state = get(browseDetailService.state);
		expect(state).toEqual(initialState);
	});

	it('close is safe to call multiple times', () => {
		browseDetailService.close();
		browseDetailService.close();

		const state = get(browseDetailService.state);
		expect(state.domain).toBeNull();
	});

	// ===== registerCallbacks / getCallbacks =====

	it('getCallbacks returns empty object initially', () => {
		const cbs = browseDetailService.getCallbacks();
		expect(cbs).toEqual({});
	});

	it('registerCallbacks stores callbacks', () => {
		const onfetch = vi.fn();
		const onclose = vi.fn();
		browseDetailService.registerCallbacks({ onfetch, onclose });

		const cbs = browseDetailService.getCallbacks();
		expect(cbs.onfetch).toBe(onfetch);
		expect(cbs.onclose).toBe(onclose);
	});

	it('registerCallbacks replaces previous callbacks', () => {
		const first = vi.fn();
		const second = vi.fn();

		browseDetailService.registerCallbacks({ onfetch: first });
		browseDetailService.registerCallbacks({ onfetch: second });

		const cbs = browseDetailService.getCallbacks();
		expect(cbs.onfetch).toBe(second);
	});

	it('close clears registered callbacks', () => {
		browseDetailService.registerCallbacks({ onfetch: vi.fn(), onclose: vi.fn() });
		browseDetailService.close();

		const cbs = browseDetailService.getCallbacks();
		expect(cbs).toEqual({});
	});

	it('registered callbacks can be invoked', () => {
		const ondownload = vi.fn();
		browseDetailService.registerCallbacks({ ondownload });

		const cbs = browseDetailService.getCallbacks();
		cbs.ondownload?.();

		expect(ondownload).toHaveBeenCalledOnce();
	});

	it('supports all callback types', () => {
		const callbacks: BrowseDetailCallbacks = {
			onfetch: vi.fn(),
			ondownload: vi.fn(),
			onstream: vi.fn(),
			onp2pstream: vi.fn(),
			onshowsearch: vi.fn(),
			onclose: vi.fn(),
			ondownloadalbum: vi.fn(),
			onaddtag: vi.fn(),
			onremovetag: vi.fn(),
			onautotag: vi.fn(),
			ondownloadaudio: vi.fn(),
			ondownloadvideo: vi.fn(),
			ontogglefavorite: vi.fn(),
			ondeleteaudio: vi.fn(),
			ondeletevideo: vi.fn()
		};
		browseDetailService.registerCallbacks(callbacks);

		const cbs = browseDetailService.getCallbacks();
		expect(Object.keys(cbs)).toHaveLength(15);
	});

	// ===== Subscription =====

	it('state is subscribable', () => {
		const values: BrowseDetailState[] = [];
		const unsub = browseDetailService.state.subscribe((v) => values.push(v));

		browseDetailService.set({ domain: 'music' });
		browseDetailService.set({ loading: true });

		unsub();

		// Initial + 2 updates
		expect(values).toHaveLength(3);
		expect(values[1].domain).toBe('music');
		expect(values[2].loading).toBe(true);
	});
});
