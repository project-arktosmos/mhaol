<script lang="ts">
	import classNames from 'classnames';
	import { onMount, onDestroy } from 'svelte';
	import { invalidateAll } from '$app/navigation';
	import { apiUrl } from '$lib/api-base';
	import { playerService } from '$services/player.service';
	import { playerAdapter } from '$adapters/classes/player.adapter';
	import { mediaDetailService } from '$services/media-detail.service';
	import { libraryService } from '$services/library.service';
	import { imageTaggerService } from '$services/image-tagger.service';
	import { modalRouterService } from '$services/modal-router.service';
	import Modal from '$components/core/Modal.svelte';
	import type { MediaDetailCardType } from '$types/media-detail.type';
	import TmdbLinkModal from '$components/libraries/TmdbLinkModal.svelte';
	import MusicBrainzLinkModal from '$components/libraries/MusicBrainzLinkModal.svelte';
	import YouTubeLinkModal from '$components/libraries/YouTubeLinkModal.svelte';
	import MediaCard from '$components/media/MediaCard.svelte';
	import MediaListCard from '$components/media/MediaListCard.svelte';
	import MediaDetail from '$components/media/MediaDetail.svelte';
	import type { MediaList, MediaListLink } from '$types/media-list.type';
	import PlayerVideo from '$components/player/PlayerVideo.svelte';
	import LyricsPanel from '$components/player/LyricsPanel.svelte';
	import type { LibraryFile } from '$types/library.type';
	import type {
		MediaItem,
		MediaItemLink,
		MediaLinkSource,
		MediaCategory
	} from '$types/media-card.type';
	import type { ImageTag, ImagesResponse } from '$types/image-tagger.type';
	import type { DisplayTMDBMovieDetails, DisplayTMDBTvShowDetails } from 'tmdb/types';
	import type { YouTubeOEmbedResponse } from 'youtube/oembed';
	import type {
		DisplayMusicBrainzRecording,
		DisplayMusicBrainzReleaseGroup
	} from 'musicbrainz/types';
	import { movieDetailsToDisplay, tvShowDetailsToDisplay } from 'tmdb/transform';
	import { releaseGroupToDisplay } from 'musicbrainz/transform';

	interface Props {
		data: {
			mediaTypes: Array<{ id: string; label: string }>;
			categories: MediaCategory[];
			linkSources: MediaLinkSource[];
			itemsByCategory: Record<string, MediaItem[]>;
			itemsByType: Record<string, MediaItem[]>;
			lists: MediaList[];
		};
	}

	const ALL_CATEGORY = '__all__';
	const ALL_TYPE = '__all_type__';
	const LISTS_TYPE = '__lists__';
	const EMPTY_TAGS: ImageTag[] = [];

	let { data }: Props = $props();

	let activeTypeId = $state(ALL_TYPE);
	let activeCategoryId = $state(ALL_CATEGORY);
	let linkModalItem: MediaItem | null = $state(null);
	let linkModalService: string | null = $state(null);
	let linkModalList: MediaList | null = $state(null);
	let linkModalListService: string | null = $state(null);
	let selectedList: MediaList | null = $state(null);
	let selectedShowGroup: { tmdbId: string; lists: MediaList[] } | null = $state(null);

	type ListGridEntry =
		| { type: 'single'; list: MediaList }
		| { type: 'show-group'; tmdbId: string; lists: MediaList[] };

	let listGridEntries: ListGridEntry[] = $derived.by(() => {
		const tmdbGroups: Record<string, MediaList[]> = {};
		const ungrouped: MediaList[] = [];

		for (const list of data.lists) {
			const links = getListLinks(list);
			const tmdbId = links.tmdb?.serviceId;
			if (tmdbId) {
				if (!tmdbGroups[tmdbId]) tmdbGroups[tmdbId] = [];
				tmdbGroups[tmdbId].push(list);
			} else {
				ungrouped.push(list);
			}
		}

		const entries: ListGridEntry[] = [];
		for (const [tmdbId, lists] of Object.entries(tmdbGroups)) {
			if (lists.length >= 2) {
				const sorted = [...lists].sort((a, b) => {
					const sa = getListLinks(a).tmdb?.seasonNumber ?? 999;
					const sb = getListLinks(b).tmdb?.seasonNumber ?? 999;
					return sa - sb;
				});
				entries.push({ type: 'show-group', tmdbId, lists: sorted });
			} else {
				entries.push({ type: 'single', list: lists[0] });
			}
		}
		for (const list of ungrouped) {
			entries.push({ type: 'single', list });
		}
		return entries;
	});

	// Track link overrides so we can update without full page reload
	let linkOverrides: Record<string, Record<string, MediaItemLink | null>> = $state({});

	// Track list link overrides
	let listLinkOverrides: Record<string, Record<string, MediaListLink | null>> = $state({});

	// List-level TMDB metadata state (keyed by tmdbId)
	let listTmdbMetadata: Record<string, DisplayTMDBTvShowDetails> = $state({});
	let listTmdbLoading: Set<string> = $state(new Set());

	// List-level MusicBrainz metadata state
	let listMbMetadata: Record<string, DisplayMusicBrainzReleaseGroup> = $state({});
	let listMbLoading: Set<string> = $state(new Set());

	// TMDB metadata state
	let tmdbMetadata: Record<string, DisplayTMDBMovieDetails | DisplayTMDBTvShowDetails> = $state({});
	let tmdbLoading: Set<string> = $state(new Set());

	// YouTube metadata state
	let youtubeMetadata: Record<string, YouTubeOEmbedResponse> = $state({});
	let youtubeLoading: Set<string> = $state(new Set());

	// MusicBrainz metadata state
	let musicbrainzMetadata: Record<string, DisplayMusicBrainzRecording> = $state({});
	let musicbrainzLoading: Set<string> = $state(new Set());

	// Merged loading state
	let metadataLoading = $derived(
		new Set([...tmdbLoading, ...youtubeLoading, ...musicbrainzLoading])
	);

	// Image tags state
	let imageTagsMap: Record<string, ImageTag[]> = $state({});

	// Scan all libraries state
	let scanning = $state(false);

	// Image tagger state for progress display
	const taggerState = imageTaggerService.state;

	async function handleScanAll() {
		scanning = true;
		try {
			await libraryService.scanAllLibraries();
			await invalidateAll();
			autoTagNewImages();
		} finally {
			scanning = false;
		}
	}

	function autoTagNewImages() {
		const allItems = Object.values(data.itemsByType).flat();
		const untaggedImageIds = allItems
			.filter(
				(item) =>
					item.mediaTypeId === 'image' &&
					(!imageTagsMap[item.id] || imageTagsMap[item.id].length === 0)
			)
			.map((item) => item.id);

		if (untaggedImageIds.length === 0) return;

		imageTaggerService.autoTagImages(untaggedImageIds, (itemId, tags) => {
			imageTagsMap = { ...imageTagsMap, [itemId]: tags };
		});
	}

	onMount(async () => {
		libraryService.initialize();
		try {
			const res = await fetch(apiUrl('/api/images'));
			if (res.ok) {
				const data: ImagesResponse = await res.json();
				const map: Record<string, ImageTag[]> = {};
				for (const img of data.images) {
					if (img.tags.length > 0) {
						map[img.id] = img.tags;
					}
				}
				imageTagsMap = map;
			}
		} catch {
			// Image tags are non-critical, fail silently
		}
	});

	function getItemLinks(item: MediaItem): Record<string, MediaItemLink> {
		const overrides = linkOverrides[item.id];
		if (!overrides) return item.links;
		const merged = { ...item.links };
		for (const [service, link] of Object.entries(overrides)) {
			if (link === null) {
				delete merged[service];
			} else {
				merged[service] = link;
			}
		}
		return merged;
	}

	let isAllType = $derived(activeTypeId === ALL_TYPE);
	let isListsType = $derived(activeTypeId === LISTS_TYPE);

	let activeType = $derived(
		activeTypeId === ALL_TYPE || activeTypeId === LISTS_TYPE
			? activeTypeId
			: activeTypeId || data.mediaTypes[0]?.id || ''
	);

	let categoriesForType = $derived(
		isAllType ? data.categories : data.categories.filter((c) => c.mediaTypeId === activeType)
	);

	let activeCategory = $derived.by(() => {
		if (activeCategoryId === ALL_CATEGORY) return ALL_CATEGORY;
		if (categoriesForType.some((c) => c.id === activeCategoryId)) return activeCategoryId;
		return ALL_CATEGORY;
	});

	let isAllCategoryView = $derived(activeCategory === ALL_CATEGORY);

	let items = $derived.by(() => {
		if (isAllType && isAllCategoryView) {
			return Object.values(data.itemsByType).flat();
		}
		if (isAllType && !isAllCategoryView) {
			return data.itemsByCategory[activeCategory] ?? [];
		}
		if (isAllCategoryView) {
			return data.itemsByType[activeType] ?? [];
		}
		return data.itemsByCategory[activeCategory] ?? [];
	});

	// Apply link overrides to items for card rendering
	let itemsWithOverrides = $derived(
		items.map((item) => {
			const overrides = linkOverrides[item.id];
			if (!overrides) return item;
			const merged = { ...item.links };
			for (const [service, link] of Object.entries(overrides)) {
				if (link === null) {
					delete merged[service];
				} else {
					merged[service] = link;
				}
			}
			return { ...item, links: merged };
		})
	);

	// Player state
	const playerState = playerService.state;

	// Media detail selection
	const mediaDetailStore = mediaDetailService.store;
	let selectedItemId = $derived($mediaDetailStore?.item.id ?? null);

	function resolveCardType(item: MediaItem): MediaDetailCardType {
		if (item.categoryId === 'movies' && item.links.tmdb) return 'movie';
		if (item.categoryId === 'tv' && item.links.tmdb) return 'tv';
		if (item.links.youtube) return 'youtube';
		if (item.mediaTypeId === 'audio') return 'audio';
		if (item.mediaTypeId === 'image') return 'image';
		return 'video';
	}

	async function handleTagImage(item: MediaItem) {
		await imageTaggerService.autoTagImages([item.id], (itemId, tags) => {
			imageTagsMap = { ...imageTagsMap, [itemId]: tags };
		});
	}

	async function handleAddTag(item: MediaItem, tag: string) {
		await imageTaggerService.addTag(item.id, tag);
		const existing = imageTagsMap[item.id] ?? EMPTY_TAGS;
		imageTagsMap = { ...imageTagsMap, [item.id]: [...existing, { tag, score: 1.0 }] };
	}

	async function handleRemoveTag(item: MediaItem, tag: string) {
		await imageTaggerService.removeTag(item.id, tag);
		const existing = imageTagsMap[item.id] ?? EMPTY_TAGS;
		imageTagsMap = { ...imageTagsMap, [item.id]: existing.filter((t) => t.tag !== tag) };
	}

	function handleSelect(item: MediaItem) {
		mediaDetailService.select({
			item,
			cardType: resolveCardType(item),
			tmdbMetadata: tmdbMetadata[item.id] ?? null,
			youtubeMetadata: youtubeMetadata[item.id] ?? null,
			musicbrainzMetadata: musicbrainzMetadata[item.id] ?? null,
			imageTags: imageTagsMap[item.id] ?? EMPTY_TAGS,
			imageTagging: $taggerState.taggingItemIds.includes(item.id),
			onplay: (i) => handlePlay(i),
			onlink: (i, service) => {
				linkModalItem = i;
				linkModalService = service;
			},
			onunlink: (i, service) => handleUnlink(i, service),
			ontagimage: (i) => handleTagImage(i),
			onaddtag: (i, tag) => handleAddTag(i, tag),
			onremovetag: (i, tag) => handleRemoveTag(i, tag)
		});
		modalRouterService.openMediaDetail(item.mediaTypeId, item.categoryId ?? '', item.id);
	}

	// Sync metadata updates into the active selection
	$effect(() => {
		const sel = $mediaDetailStore;
		if (!sel) return;
		const id = sel.item.id;
		const updatedItem = itemsWithOverrides.find((i) => i.id === id);
		if (!updatedItem) return;
		const newTmdb = tmdbMetadata[id] ?? null;
		const newYt = youtubeMetadata[id] ?? null;
		const newMb = musicbrainzMetadata[id] ?? null;
		const newTags = imageTagsMap[id] ?? EMPTY_TAGS;
		const newTagging = $taggerState.taggingItemIds.includes(id);
		if (
			newTmdb !== sel.tmdbMetadata ||
			newYt !== sel.youtubeMetadata ||
			newMb !== sel.musicbrainzMetadata ||
			updatedItem !== sel.item ||
			newTags !== sel.imageTags ||
			newTagging !== sel.imageTagging
		) {
			mediaDetailService.select({
				...sel,
				item: updatedItem,
				cardType: resolveCardType(updatedItem),
				tmdbMetadata: newTmdb,
				youtubeMetadata: newYt,
				musicbrainzMetadata: newMb,
				imageTags: newTags,
				imageTagging: newTagging
			});
		}
	});

	function closeMediaDetail() {
		playerService.stop();
		mediaDetailService.clear();
		modalRouterService.closeMediaDetail();
	}

	onDestroy(() => {
		mediaDetailService.clear();
	});

	// Deep-link restoration: open media detail from URL params on load
	const routerStore = modalRouterService.store;
	let deepLinkRestored = $state(false);

	$effect(() => {
		const detail = $routerStore.mediaDetail;
		if (!detail || deepLinkRestored) return;
		deepLinkRestored = true;
		const allItems = Object.values(data.itemsByType).flat();
		const item = allItems.find((i) => i.id === detail.id);
		if (item) {
			handleSelect(item);
		}
	});

	// Sync router popstate back to media detail
	$effect(() => {
		const detail = $routerStore.mediaDetail;
		if (!detail && $mediaDetailStore) {
			mediaDetailService.clear();
		}
	});

	function selectType(id: string) {
		activeTypeId = id;
		activeCategoryId = ALL_CATEGORY;
		selectedList = null;
		selectedShowGroup = null;
		closeMediaDetail();
	}

	function selectCategory(id: string) {
		activeCategoryId = id;
		closeMediaDetail();
	}

	function updateItemLinks(itemId: string, service: string, link: MediaItemLink | null) {
		linkOverrides = {
			...linkOverrides,
			[itemId]: {
				...linkOverrides[itemId],
				[service]: link
			}
		};
	}

	async function handleLink(
		tmdbId: number,
		seasonNumber: number | null,
		episodeNumber: number | null,
		type: 'movie' | 'tv'
	) {
		if (!linkModalItem) return;
		const item = linkModalItem;

		const res = await fetch(apiUrl(`/api/libraries/${item.libraryId}/items/${item.id}/tmdb`), {
			method: 'PUT',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ tmdbId, seasonNumber, episodeNumber })
		});

		if (res.ok) {
			const categoryId = type === 'movie' ? 'movies' : 'tv';
			const needsCategoryUpdate = item.categoryId !== categoryId;

			// Set categoryId before triggering reactive update so cardType routes correctly
			item.categoryId = categoryId;

			updateItemLinks(item.id, 'tmdb', {
				serviceId: String(tmdbId),
				seasonNumber,
				episodeNumber
			});

			if (needsCategoryUpdate) {
				fetch(apiUrl(`/api/libraries/${item.libraryId}/items/${item.id}/category`), {
					method: 'PUT',
					headers: { 'Content-Type': 'application/json' },
					body: JSON.stringify({ categoryId })
				});
			}
		}

		linkModalItem = null;
		linkModalService = null;
	}

	async function handleMusicBrainzLink(musicbrainzId: string) {
		if (!linkModalItem) return;
		const item = linkModalItem;

		const res = await fetch(
			apiUrl(`/api/libraries/${item.libraryId}/items/${item.id}/musicbrainz`),
			{
				method: 'PUT',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ musicbrainzId })
			}
		);

		if (res.ok) {
			updateItemLinks(item.id, 'musicbrainz', {
				serviceId: musicbrainzId,
				seasonNumber: null,
				episodeNumber: null
			});
		}

		linkModalItem = null;
		linkModalService = null;
	}

	async function handleYoutubeLink(youtubeId: string) {
		if (!linkModalItem) return;
		const item = linkModalItem;

		const res = await fetch(apiUrl(`/api/libraries/${item.libraryId}/items/${item.id}/youtube`), {
			method: 'PUT',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ youtubeId })
		});

		if (res.ok) {
			updateItemLinks(item.id, 'youtube', {
				serviceId: youtubeId,
				seasonNumber: null,
				episodeNumber: null
			});
		}

		linkModalItem = null;
		linkModalService = null;
	}

	async function handleUnlink(item: MediaItem, service: string) {
		const res = await fetch(
			apiUrl(`/api/libraries/${item.libraryId}/items/${item.id}/${service}`),
			{
				method: 'DELETE'
			}
		);

		if (res.ok) {
			updateItemLinks(item.id, service, null);
			if (service === 'tmdb') {
				const { [item.id]: _, ...rest } = tmdbMetadata;
				tmdbMetadata = rest;
			}
			if (service === 'youtube') {
				const { [item.id]: _, ...rest } = youtubeMetadata;
				youtubeMetadata = rest;
			}
			if (service === 'musicbrainz') {
				const { [item.id]: _, ...rest } = musicbrainzMetadata;
				musicbrainzMetadata = rest;
			}
		}
	}

	function handlePlay(item: MediaItem) {
		const playableFile = playerAdapter.fromMediaItem(item);
		playerService.play(playableFile);
	}

	function getListLinks(list: MediaList): Record<string, MediaListLink> {
		const overrides = listLinkOverrides[list.id];
		if (!overrides) return list.links;
		const merged = { ...list.links };
		for (const [service, link] of Object.entries(overrides)) {
			if (link === null) {
				delete merged[service];
			} else {
				merged[service] = link;
			}
		}
		return merged;
	}

	function listAsLibraryFile(list: MediaList): LibraryFile {
		return {
			id: list.id,
			name: list.title,
			path: '',
			extension: '',
			mediaType: list.mediaType as LibraryFile['mediaType'],
			categoryId: null,
			links: {}
		};
	}

	async function handleListTmdbLink(
		tmdbId: number,
		_seasonNumber: number | null,
		_episodeNumber: number | null,
		_type: 'movie' | 'tv'
	) {
		if (!linkModalList) return;
		const list = linkModalList;
		const res = await fetch(apiUrl(`/api/media-lists/${list.id}/tmdb`), {
			method: 'PUT',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ tmdbId })
		});
		if (res.ok) {
			const tmdbIdStr = String(tmdbId);
			listLinkOverrides = {
				...listLinkOverrides,
				[list.id]: {
					...listLinkOverrides[list.id],
					tmdb: { serviceId: tmdbIdStr, seasonNumber: null }
				}
			};
			// Clear metadata for this tmdbId so it gets re-fetched
			const { [tmdbIdStr]: _, ...restTmdb } = listTmdbMetadata;
			listTmdbMetadata = restTmdb;
		}
		linkModalList = null;
		linkModalListService = null;
	}

	async function handleListMusicBrainzLink(musicbrainzId: string) {
		if (!linkModalList) return;
		const list = linkModalList;
		const res = await fetch(apiUrl(`/api/media-lists/${list.id}/musicbrainz`), {
			method: 'PUT',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ musicbrainzId })
		});
		if (res.ok) {
			listLinkOverrides = {
				...listLinkOverrides,
				[list.id]: {
					...listLinkOverrides[list.id],
					musicbrainz: { serviceId: musicbrainzId, seasonNumber: null }
				}
			};
			const { [list.id]: _, ...restMb } = listMbMetadata;
			listMbMetadata = restMb;
		}
		linkModalList = null;
		linkModalListService = null;
	}

	async function handleListUnlink(list: MediaList, service: string) {
		const links = getListLinks(list);
		const res = await fetch(apiUrl(`/api/media-lists/${list.id}/${service}`), {
			method: 'DELETE'
		});
		if (res.ok) {
			listLinkOverrides = {
				...listLinkOverrides,
				[list.id]: { ...listLinkOverrides[list.id], [service]: null }
			};
			if (service === 'musicbrainz') {
				const { [list.id]: _, ...rest } = listMbMetadata;
				listMbMetadata = rest;
			}
			// TMDB metadata keyed by tmdbId — don't remove since other lists may share it
		}
	}

	async function handleListSetSeason(list: MediaList, seasonNumber: number | null) {
		const links = getListLinks(list);
		const tmdbLink = links.tmdb;
		if (!tmdbLink) return;
		const res = await fetch(apiUrl(`/api/media-lists/${list.id}/tmdb`), {
			method: 'PUT',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ tmdbId: Number(tmdbLink.serviceId), seasonNumber })
		});
		if (res.ok) {
			listLinkOverrides = {
				...listLinkOverrides,
				[list.id]: {
					...listLinkOverrides[list.id],
					tmdb: { serviceId: tmdbLink.serviceId, seasonNumber }
				}
			};
		}
	}

	async function fetchListTmdbMetadata(tmdbId: string) {
		if (listTmdbMetadata[tmdbId] || listTmdbLoading.has(tmdbId)) return;
		listTmdbLoading = new Set([...listTmdbLoading, tmdbId]);
		try {
			const res = await fetch(apiUrl(`/api/tmdb/tv/${tmdbId}`));
			if (res.ok) {
				const raw = await res.json();
				listTmdbMetadata = { ...listTmdbMetadata, [tmdbId]: tvShowDetailsToDisplay(raw) };
			}
		} catch (e) {
			console.error('Failed to load list TMDB metadata:', e);
		} finally {
			const next = new Set(listTmdbLoading);
			next.delete(tmdbId);
			listTmdbLoading = next;
		}
	}

	async function fetchListMbMetadata(listId: string, mbId: string) {
		if (listMbMetadata[listId] || listMbLoading.has(listId)) return;
		listMbLoading = new Set([...listMbLoading, listId]);
		try {
			const res = await fetch(apiUrl(`/api/musicbrainz/release-group/${mbId}`));
			if (res.ok) {
				const raw = await res.json();
				listMbMetadata = { ...listMbMetadata, [listId]: releaseGroupToDisplay(raw) };
			}
		} catch (e) {
			console.error('Failed to load list MusicBrainz metadata:', e);
		} finally {
			const next = new Set(listMbLoading);
			next.delete(listId);
			listMbLoading = next;
		}
	}

	function itemAsLibraryFile(item: MediaItem): LibraryFile {
		return {
			id: item.id,
			name: item.name,
			path: item.path,
			extension: item.extension,
			mediaType: item.mediaTypeId as LibraryFile['mediaType'],
			categoryId: item.categoryId,
			links: getItemLinks(item)
		};
	}

	async function fetchTmdbMetadata(item: MediaItem) {
		const tmdbLink = item.links.tmdb;
		if (!tmdbLink || tmdbMetadata[item.id] || tmdbLoading.has(item.id)) return;

		tmdbLoading = new Set([...tmdbLoading, item.id]);

		const isTv = item.categoryId === 'tv';
		const endpoint = isTv
			? `/api/tmdb/tv/${tmdbLink.serviceId}`
			: `/api/tmdb/movies/${tmdbLink.serviceId}`;

		try {
			const res = await fetch(apiUrl(endpoint));
			if (res.ok) {
				const data = await res.json();
				tmdbMetadata[item.id] = isTv ? tvShowDetailsToDisplay(data) : movieDetailsToDisplay(data);
			}
		} catch (e) {
			console.error('Failed to load TMDB metadata:', e);
		} finally {
			const next = new Set(tmdbLoading);
			next.delete(item.id);
			tmdbLoading = next;
		}
	}

	async function fetchYoutubeMetadata(item: MediaItem) {
		const youtubeLink = item.links.youtube;
		if (!youtubeLink || youtubeMetadata[item.id] || youtubeLoading.has(item.id)) return;

		youtubeLoading = new Set([...youtubeLoading, item.id]);

		try {
			const res = await fetch(apiUrl(`/api/youtube/oembed?videoId=${youtubeLink.serviceId}`));
			if (res.ok) {
				youtubeMetadata[item.id] = await res.json();
			}
		} catch (e) {
			console.error('Failed to load YouTube metadata:', e);
		} finally {
			const next = new Set(youtubeLoading);
			next.delete(item.id);
			youtubeLoading = next;
		}
	}

	async function fetchMusicbrainzMetadata(item: MediaItem) {
		const mbLink = item.links.musicbrainz;
		if (!mbLink || musicbrainzMetadata[item.id] || musicbrainzLoading.has(item.id)) return;

		musicbrainzLoading = new Set([...musicbrainzLoading, item.id]);

		try {
			const res = await fetch(apiUrl(`/api/musicbrainz/recording/${mbLink.serviceId}`));
			if (res.ok) {
				musicbrainzMetadata[item.id] = await res.json();
			}
		} catch (e) {
			console.error('Failed to load MusicBrainz metadata:', e);
		} finally {
			const next = new Set(musicbrainzLoading);
			next.delete(item.id);
			musicbrainzLoading = next;
		}
	}

	// Pre-fetched lyrics items tracker (prevents duplicate fetches)
	let lyricsFetched: Set<string> = $state(new Set());

	async function fetchLyrics(item: MediaItem) {
		if (!item.links.musicbrainz || lyricsFetched.has(item.id)) return;
		lyricsFetched = new Set([...lyricsFetched, item.id]);

		try {
			await fetch(apiUrl(`/api/lyrics/${item.id}`));
		} catch {
			// Lyrics pre-fetch is best-effort
		}
	}

	$effect(() => {
		for (const item of itemsWithOverrides) {
			if (item.links.tmdb) {
				fetchTmdbMetadata(item);
			}
			if (item.links.youtube) {
				fetchYoutubeMetadata(item);
			}
			if (item.links.musicbrainz) {
				fetchMusicbrainzMetadata(item);
				if (item.mediaTypeId === 'audio') {
					fetchLyrics(item);
				}
			}
		}
	});

	// Fetch metadata for linked lists
	$effect(() => {
		for (const list of data.lists) {
			const links = getListLinks(list);
			if (links.tmdb) {
				fetchListTmdbMetadata(links.tmdb.serviceId);
			}
			if (links.musicbrainz) {
				fetchListMbMetadata(list.id, links.musicbrainz.serviceId);
			}
		}
	});
</script>

<div class="container mx-auto p-4">
	<div class="mb-6 flex items-center justify-between">
		<div>
			<h1 class="text-3xl font-bold">Media</h1>
			<p class="text-sm opacity-70">Browse your media library</p>
		</div>
		<button class="btn btn-sm btn-accent" onclick={handleScanAll} disabled={scanning}>
			{#if scanning}
				<span class="loading loading-xs loading-spinner"></span>
			{:else}
				Scan
			{/if}
		</button>
	</div>

	<!-- Tier 1: All + Media Types -->
	<div class="mb-3 flex flex-wrap gap-2">
		<button
			class={classNames('btn btn-sm', {
				'btn-primary': isAllType,
				'btn-ghost': !isAllType
			})}
			onclick={() => selectType(ALL_TYPE)}
		>
			All
		</button>
		<button
			class={classNames('btn btn-sm', {
				'btn-primary': isListsType,
				'btn-ghost': !isListsType
			})}
			onclick={() => selectType(LISTS_TYPE)}
		>
			Lists
		</button>
		{#each data.mediaTypes as type}
			<button
				class={classNames('btn btn-sm', {
					'btn-primary': activeType === type.id,
					'btn-ghost': activeType !== type.id
				})}
				onclick={() => selectType(type.id)}
			>
				{type.label}
			</button>
		{/each}
	</div>

	<!-- Tier 2: All + Categories for selected type (hidden when Lists tab active) -->
	{#if !isListsType && categoriesForType.length > 0}
		<div class="mb-6 flex flex-wrap gap-2">
			<button
				class={classNames('btn btn-xs', {
					'btn-secondary': isAllCategoryView,
					'btn-ghost': !isAllCategoryView
				})}
				onclick={() => selectCategory(ALL_CATEGORY)}
			>
				All
			</button>
			{#each categoriesForType as category}
				<button
					class={classNames('btn btn-xs', {
						'btn-secondary': activeCategory === category.id,
						'btn-ghost': activeCategory !== category.id
					})}
					onclick={() => selectCategory(category.id)}
				>
					{category.label}
				</button>
			{/each}
		</div>
	{/if}

	{#if isListsType}
		<!-- Lists view -->
		{#if selectedList}
			{@const currentListLinks = getListLinks(selectedList)}
			{@const listTmdb = currentListLinks.tmdb
				? (listTmdbMetadata[currentListLinks.tmdb.serviceId] ?? null)
				: null}
			{@const listMb = listMbMetadata[selectedList.id] ?? null}
			{@const selectedSeason = currentListLinks.tmdb?.seasonNumber ?? null}
			{@const seasonMeta =
				listTmdb && selectedSeason != null
					? (listTmdb.seasons.find((s) => s.seasonNumber === selectedSeason) ?? null)
					: null}
			<div class="mb-4 flex items-center gap-2">
				<button
					class="btn btn-ghost btn-sm"
					onclick={() => {
						selectedList = null;
					}}
				>
					&larr; Back
				</button>
				<h2 class="text-xl font-semibold">{selectedList.title}</h2>
				<div class="ml-auto flex gap-2">
					{#if selectedList.mediaType === 'video'}
						{#if currentListLinks.tmdb}
							<button
								class="btn btn-outline btn-xs btn-error"
								onclick={() => handleListUnlink(selectedList!, 'tmdb')}
							>
								Unlink TV Show
							</button>
						{:else}
							<button
								class="btn btn-outline btn-xs"
								onclick={() => {
									linkModalList = selectedList;
									linkModalListService = 'tmdb';
								}}
							>
								Link TV Show
							</button>
						{/if}
					{/if}
					{#if selectedList.mediaType === 'audio'}
						{#if currentListLinks.musicbrainz}
							<button
								class="btn btn-outline btn-xs btn-error"
								onclick={() => handleListUnlink(selectedList!, 'musicbrainz')}
							>
								Unlink Album
							</button>
						{:else}
							<button
								class="btn btn-outline btn-xs"
								onclick={() => {
									linkModalList = selectedList;
									linkModalListService = 'musicbrainz';
								}}
							>
								Link Album
							</button>
						{/if}
					{/if}
				</div>
			</div>
			{#if listTmdb}
				<div class="mb-4 flex gap-4 rounded-lg bg-base-200 p-4">
					{#if seasonMeta?.posterUrl}
						<img
							src={seasonMeta.posterUrl}
							alt={seasonMeta.name}
							class="h-40 w-auto rounded-lg object-cover"
						/>
					{:else if listTmdb.posterUrl}
						<img
							src={listTmdb.posterUrl}
							alt={listTmdb.name}
							class="h-40 w-auto rounded-lg object-cover"
						/>
					{/if}
					<div class="flex flex-1 flex-col gap-1">
						<h3 class="text-lg font-semibold">{listTmdb.name}</h3>
						{#if seasonMeta}
							<p class="text-sm font-medium opacity-70">{seasonMeta.name}</p>
						{/if}
						<p class="text-xs opacity-50">
							{listTmdb.firstAirYear}{listTmdb.lastAirYear ? `–${listTmdb.lastAirYear}` : ''}
						</p>
						<p class="line-clamp-3 text-sm opacity-70">
							{seasonMeta?.overview || listTmdb.overview}
						</p>
						{#if listTmdb.seasons.length > 0}
							<div class="mt-auto">
								<select
									class="select-bordered select select-xs"
									value={selectedSeason ?? ''}
									onchange={(e) => {
										const val = (e.target as HTMLSelectElement).value;
										handleListSetSeason(selectedList!, val === '' ? null : Number(val));
									}}
								>
									<option value="">All Seasons</option>
									{#each listTmdb.seasons as season}
										<option value={season.seasonNumber}>{season.name}</option>
									{/each}
								</select>
							</div>
						{/if}
					</div>
				</div>
			{:else if listMb}
				<div class="mb-4 flex gap-4 rounded-lg bg-base-200 p-4">
					{#if listMb.coverArtUrl}
						<img
							src={listMb.coverArtUrl}
							alt={listMb.title}
							class="h-40 w-40 rounded-lg object-cover"
						/>
					{/if}
					<div class="flex flex-1 flex-col gap-1">
						<h3 class="text-lg font-semibold">{listMb.title}</h3>
						<p class="text-sm opacity-70">{listMb.artistCredits}</p>
						{#if listMb.firstReleaseYear && listMb.firstReleaseYear !== 'Unknown'}
							<p class="text-xs opacity-50">{listMb.firstReleaseYear}</p>
						{/if}
						{#if listMb.primaryType}
							<span class="mt-1 badge badge-outline badge-sm">{listMb.primaryType}</span>
						{/if}
					</div>
				</div>
			{/if}
			{#if selectedList.items.length > 0}
				<div
					class="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6"
				>
					{#each selectedList.items as item (item.id)}
						<MediaCard
							{item}
							tmdbMetadata={tmdbMetadata[item.id] ?? null}
							youtubeMetadata={youtubeMetadata[item.id] ?? null}
							musicbrainzMetadata={musicbrainzMetadata[item.id] ?? null}
							metadataLoading={metadataLoading.has(item.id)}
							imageTags={imageTagsMap[item.id] ?? EMPTY_TAGS}
							tagging={$taggerState.taggingItemIds.includes(item.id)}
							selected={selectedItemId === item.id}
							onselect={(i) => handleSelect(i)}
						/>
					{/each}
				</div>
			{:else}
				<div class="rounded-lg bg-base-200 p-8 text-center">
					<p class="opacity-50">No items in this list.</p>
				</div>
			{/if}
		{:else if selectedShowGroup}
			{@const tmdbMeta = listTmdbMetadata[selectedShowGroup.tmdbId] ?? null}
			<div class="mb-4 flex items-center gap-2">
				<button
					class="btn btn-ghost btn-sm"
					onclick={() => {
						selectedShowGroup = null;
					}}
				>
					&larr; Back
				</button>
				<h2 class="text-xl font-semibold">{tmdbMeta?.name ?? 'TV Show'}</h2>
			</div>
			{#if tmdbMeta}
				<div class="mb-6 flex gap-4 rounded-lg bg-base-200 p-4">
					{#if tmdbMeta.posterUrl}
						<img
							src={tmdbMeta.posterUrl}
							alt={tmdbMeta.name}
							class="h-40 w-auto rounded-lg object-cover"
						/>
					{/if}
					<div class="flex flex-1 flex-col gap-1">
						<h3 class="text-lg font-semibold">{tmdbMeta.name}</h3>
						<p class="text-xs opacity-50">
							{tmdbMeta.firstAirYear}{tmdbMeta.lastAirYear ? `–${tmdbMeta.lastAirYear}` : ''}
						</p>
						<p class="line-clamp-3 text-sm opacity-70">{tmdbMeta.overview}</p>
					</div>
				</div>
			{/if}
			{#each selectedShowGroup.lists as list (list.id)}
				{@const links = getListLinks(list)}
				{@const seasonNum = links.tmdb?.seasonNumber}
				{@const seasonMeta =
					tmdbMeta && seasonNum != null
						? (tmdbMeta.seasons.find((s) => s.seasonNumber === seasonNum) ?? null)
						: null}
				<div class="mb-6">
					<div class="mb-3 flex items-center gap-3">
						{#if seasonMeta?.posterUrl}
							<img
								src={seasonMeta.posterUrl}
								alt={seasonMeta.name}
								class="h-16 w-auto rounded object-cover"
							/>
						{/if}
						<div>
							<h3 class="text-base font-semibold">
								{seasonMeta?.name ?? list.title}
							</h3>
							{#if seasonMeta?.episodeCount}
								<p class="text-xs opacity-50">{seasonMeta.episodeCount} episodes</p>
							{/if}
						</div>
					</div>
					{#if list.items.length > 0}
						<div
							class="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6"
						>
							{#each list.items as item (item.id)}
								<MediaCard
									{item}
									tmdbMetadata={tmdbMetadata[item.id] ?? null}
									youtubeMetadata={youtubeMetadata[item.id] ?? null}
									musicbrainzMetadata={musicbrainzMetadata[item.id] ?? null}
									metadataLoading={metadataLoading.has(item.id)}
									imageTags={imageTagsMap[item.id] ?? EMPTY_TAGS}
									tagging={$taggerState.taggingItemIds.includes(item.id)}
									selected={selectedItemId === item.id}
									onselect={(i) => handleSelect(i)}
								/>
							{/each}
						</div>
					{:else}
						<p class="text-sm opacity-50">No items in this season.</p>
					{/if}
				</div>
			{/each}
		{:else if listGridEntries.length > 0}
			<div
				class="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6"
			>
				{#each listGridEntries as entry}
					{#if entry.type === 'single'}
						{@const list = entry.list}
						{@const links = getListLinks(list)}
						<MediaListCard
							{list}
							tmdbMetadata={links.tmdb ? (listTmdbMetadata[links.tmdb.serviceId] ?? null) : null}
							mbMetadata={listMbMetadata[list.id] ?? null}
							onselect={(l) => {
								selectedList = l;
							}}
						/>
					{:else}
						{@const tmdbMeta = listTmdbMetadata[entry.tmdbId] ?? null}
						{@const representative = entry.lists[0]}
						<MediaListCard
							list={representative}
							tmdbMetadata={tmdbMeta}
							seasonCount={entry.lists.length}
							onselect={() => {
								selectedShowGroup = { tmdbId: entry.tmdbId, lists: entry.lists };
							}}
						/>
					{/if}
				{/each}
			</div>
		{:else}
			<div class="rounded-lg bg-base-200 p-8 text-center">
				<p class="opacity-50">
					No lists yet. Scan a library with directories containing multiple audio or video files.
				</p>
			</div>
		{/if}
	{:else}
		<!-- Items grid -->
		{#if itemsWithOverrides.length > 0}
			<div
				class="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6"
			>
				{#each itemsWithOverrides as item (item.id)}
					<MediaCard
						{item}
						tmdbMetadata={tmdbMetadata[item.id] ?? null}
						youtubeMetadata={youtubeMetadata[item.id] ?? null}
						musicbrainzMetadata={musicbrainzMetadata[item.id] ?? null}
						metadataLoading={metadataLoading.has(item.id)}
						imageTags={imageTagsMap[item.id] ?? EMPTY_TAGS}
						tagging={$taggerState.taggingItemIds.includes(item.id)}
						selected={selectedItemId === item.id}
						onselect={(i) => handleSelect(i)}
					/>
				{/each}
			</div>
		{:else}
			<div class="rounded-lg bg-base-200 p-8 text-center">
				<p class="opacity-50">No items in this category.</p>
			</div>
		{/if}
	{/if}
</div>

<Modal open={!!$mediaDetailStore} maxWidth="max-w-lg" onclose={closeMediaDetail}>
	{#if $mediaDetailStore}
		<MediaDetail selection={$mediaDetailStore} onclose={closeMediaDetail} />
		{#if $playerState.currentFile && $playerState.currentFile.id !== $mediaDetailStore?.item.id}
			<div class="mt-4 border-t border-base-300 pt-4">
				<div class="mb-2 flex items-center justify-between">
					<h2 class="text-sm font-semibold tracking-wide text-base-content/50 uppercase">
						Now Playing
					</h2>
					<button
						class="btn btn-square btn-ghost btn-xs"
						onclick={() => playerService.stop()}
						aria-label="Close player"
					>
						&times;
					</button>
				</div>
				<p class="mb-2 truncate text-xs opacity-60" title={$playerState.currentFile.name}>
					{$playerState.currentFile.name}
				</p>
				<PlayerVideo
					file={$playerState.currentFile}
					connectionState={$playerState.connectionState}
					positionSecs={$playerState.positionSecs}
					durationSecs={$playerState.durationSecs}
				/>
				{#if $playerState.currentFile.mode === 'audio'}
					<div class="mt-2">
						<LyricsPanel
							currentFile={$playerState.currentFile}
							positionSecs={$playerState.positionSecs}
							on:seek={(e) => playerService.seek(e.detail.positionSecs)}
						/>
					</div>
				{/if}
			</div>
		{/if}
	{/if}
</Modal>

{#if linkModalItem && linkModalService === 'tmdb-movie'}
	<TmdbLinkModal
		file={itemAsLibraryFile(linkModalItem)}
		type="movie"
		onlink={handleLink}
		onclose={() => {
			linkModalItem = null;
			linkModalService = null;
		}}
	/>
{/if}

{#if linkModalItem && linkModalService === 'tmdb-tv'}
	<TmdbLinkModal
		file={itemAsLibraryFile(linkModalItem)}
		type="tv"
		onlink={handleLink}
		onclose={() => {
			linkModalItem = null;
			linkModalService = null;
		}}
	/>
{/if}

{#if linkModalItem && linkModalService === 'musicbrainz'}
	<MusicBrainzLinkModal
		file={itemAsLibraryFile(linkModalItem)}
		onlink={handleMusicBrainzLink}
		onclose={() => {
			linkModalItem = null;
			linkModalService = null;
		}}
	/>
{/if}

{#if linkModalItem && linkModalService === 'youtube'}
	<YouTubeLinkModal
		file={itemAsLibraryFile(linkModalItem)}
		onlink={handleYoutubeLink}
		onclose={() => {
			linkModalItem = null;
			linkModalService = null;
		}}
	/>
{/if}

{#if linkModalList && linkModalListService === 'tmdb'}
	<TmdbLinkModal
		file={listAsLibraryFile(linkModalList)}
		type="tv"
		onlink={handleListTmdbLink}
		onclose={() => {
			linkModalList = null;
			linkModalListService = null;
		}}
	/>
{/if}

{#if linkModalList && linkModalListService === 'musicbrainz'}
	<MusicBrainzLinkModal
		file={listAsLibraryFile(linkModalList)}
		onlink={handleListMusicBrainzLink}
		onclose={() => {
			linkModalList = null;
			linkModalListService = null;
		}}
	/>
{/if}
