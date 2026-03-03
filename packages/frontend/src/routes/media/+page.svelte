<script lang="ts">
	import classNames from 'classnames';
	import { onMount, onDestroy } from 'svelte';
	import { invalidateAll } from '$app/navigation';
	import { apiUrl } from '$lib/api-base';
	import { playerService } from '$services/player.service';
	import { playerAdapter } from '$adapters/classes/player.adapter';
	import { mediaDetailService } from '$services/media-detail.service';
	import { torrentModalService } from '$services/torrent-modal.service';
	import { youtubeModalService } from '$services/youtube-modal.service';
	import { downloadsService } from '$services/downloads.service';
	import { libraryService } from '$services/library.service';
	import type { MediaDetailCardType } from '$types/media-detail.type';
	import TmdbLinkModal from '$components/libraries/TmdbLinkModal.svelte';
	import MusicBrainzLinkModal from '$components/libraries/MusicBrainzLinkModal.svelte';
	import MediaCard from '$components/media/MediaCard.svelte';
	import type { LibraryFile } from '$types/library.type';
	import type { MediaItem, MediaItemLink, MediaLinkSource, MediaCategory } from '$types/media-card.type';
	import type { ImageTag, ImagesResponse } from '$types/image-tagger.type';
	import type { DisplayTMDBMovieDetails, DisplayTMDBTvShowDetails } from 'tmdb/types';
	import type { YouTubeOEmbedResponse } from 'youtube/oembed';
	import type { DisplayMusicBrainzRecording } from 'musicbrainz/types';
	import { movieDetailsToDisplay, tvShowDetailsToDisplay } from 'tmdb/transform';

	interface Props {
		data: {
			mediaTypes: Array<{ id: string; label: string }>;
			categories: MediaCategory[];
			linkSources: MediaLinkSource[];
			itemsByCategory: Record<string, MediaItem[]>;
			itemsByType: Record<string, MediaItem[]>;
		};
	}

	const ALL_CATEGORY = '__all__';
	const ALL_TYPE = '__all_type__';

	let { data }: Props = $props();

	let activeTypeId = $state(ALL_TYPE);
	let activeCategoryId = $state(ALL_CATEGORY);
	let linkModalItem: MediaItem | null = $state(null);
	let linkModalService: string | null = $state(null);

	// Track link overrides so we can update without full page reload
	let linkOverrides: Record<string, Record<string, MediaItemLink | null>> = $state({});

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
	let metadataLoading = $derived(new Set([...tmdbLoading, ...youtubeLoading, ...musicbrainzLoading]));

	// Image tags state
	let imageTagsMap: Record<string, ImageTag[]> = $state({});

	// Scan all libraries state
	let scanning = $state(false);

	async function handleScanAll() {
		scanning = true;
		try {
			await libraryService.scanAllLibraries();
			await invalidateAll();
		} finally {
			scanning = false;
		}
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

	let activeType = $derived(activeTypeId === ALL_TYPE ? ALL_TYPE : (activeTypeId || data.mediaTypes[0]?.id || ''));

	let categoriesForType = $derived(
		isAllType
			? data.categories
			: data.categories.filter((c) => c.mediaTypeId === activeType)
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

	// Media detail selection
	const mediaDetailStore = mediaDetailService.store;
	let selectedItemId = $derived(($mediaDetailStore)?.item.id ?? null);

	function resolveCardType(item: MediaItem): MediaDetailCardType {
		if (item.categoryId === 'movies' && item.links.tmdb) return 'movie';
		if (item.categoryId === 'tv' && item.links.tmdb) return 'tv';
		if (item.links.youtube) return 'youtube';
		if (item.mediaTypeId === 'audio') return 'audio';
		if (item.mediaTypeId === 'image') return 'image';
		return 'video';
	}

	function handleSelect(item: MediaItem) {
		mediaDetailService.select({
			item,
			cardType: resolveCardType(item),
			tmdbMetadata: tmdbMetadata[item.id] ?? null,
			youtubeMetadata: youtubeMetadata[item.id] ?? null,
			musicbrainzMetadata: musicbrainzMetadata[item.id] ?? null,
			imageTags: imageTagsMap[item.id] ?? [],
			onplay: (i) => handlePlay(i),
			onlink: (i, service) => { linkModalItem = i; linkModalService = service; },
			onunlink: (i, service) => handleUnlink(i, service)
		});
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
		const newTags = imageTagsMap[id] ?? [];
		if (newTmdb !== sel.tmdbMetadata || newYt !== sel.youtubeMetadata || newMb !== sel.musicbrainzMetadata || updatedItem !== sel.item) {
			mediaDetailService.select({
				...sel,
				item: updatedItem,
				cardType: resolveCardType(updatedItem),
				tmdbMetadata: newTmdb,
				youtubeMetadata: newYt,
				musicbrainzMetadata: newMb,
				imageTags: newTags
			});
		}
	});

	onDestroy(() => {
		mediaDetailService.clear();
	});

	function selectType(id: string) {
		activeTypeId = id;
		activeCategoryId = ALL_CATEGORY;
		mediaDetailService.clear();
	}

	function selectCategory(id: string) {
		activeCategoryId = id;
		mediaDetailService.clear();
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

	async function handleLink(tmdbId: number, seasonNumber: number | null, episodeNumber: number | null, type: 'movie' | 'tv') {
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

		const res = await fetch(apiUrl(`/api/libraries/${item.libraryId}/items/${item.id}/musicbrainz`), {
			method: 'PUT',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ musicbrainzId })
		});

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

	async function handleUnlink(item: MediaItem, service: string) {
		const res = await fetch(apiUrl(`/api/libraries/${item.libraryId}/items/${item.id}/${service}`), {
			method: 'DELETE'
		});

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
				tmdbMetadata[item.id] = isTv
					? tvShowDetailsToDisplay(data)
					: movieDetailsToDisplay(data);
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
</script>

<div class="container mx-auto p-4">
	<div class="mb-6 flex items-center justify-between">
		<div>
			<h1 class="text-3xl font-bold">Media</h1>
			<p class="text-sm opacity-70">Browse your media library</p>
		</div>
		<div class="flex gap-2">
			<button class="btn btn-sm btn-accent" onclick={handleScanAll} disabled={scanning}>
				{#if scanning}
					<span class="loading loading-spinner loading-xs"></span>
				{:else}
					Scan
				{/if}
			</button>
			<button class="btn btn-sm btn-info" onclick={() => youtubeModalService.open()}>
				YouTube
			</button>
			<button class="btn btn-sm btn-primary" onclick={() => torrentModalService.open()}>
				Torrent
			</button>
			<button class="btn btn-sm btn-secondary" onclick={() => downloadsService.openModal()}>
				Downloads
			</button>
		</div>
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

	<!-- Tier 2: All + Categories for selected type -->
	{#if categoriesForType.length > 0}
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

	<!-- Items grid -->
	{#if itemsWithOverrides.length > 0}
		<div class="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6">
			{#each itemsWithOverrides as item (item.id)}
				<MediaCard
					{item}
					tmdbMetadata={tmdbMetadata[item.id] ?? null}
					youtubeMetadata={youtubeMetadata[item.id] ?? null}
					musicbrainzMetadata={musicbrainzMetadata[item.id] ?? null}
					metadataLoading={metadataLoading.has(item.id)}
					imageTags={imageTagsMap[item.id] ?? []}
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
</div>

{#if linkModalItem && linkModalService === 'tmdb'}
	<TmdbLinkModal
		file={itemAsLibraryFile(linkModalItem)}
		onlink={handleLink}
		onclose={() => { linkModalItem = null; linkModalService = null; }}
	/>
{/if}

{#if linkModalItem && linkModalService === 'musicbrainz'}
	<MusicBrainzLinkModal
		file={itemAsLibraryFile(linkModalItem)}
		onlink={handleMusicBrainzLink}
		onclose={() => { linkModalItem = null; linkModalService = null; }}
	/>
{/if}
