<script lang="ts">
	import classNames from 'classnames';
	import { onMount, onDestroy } from 'svelte';
	import { invalidateAll } from '$app/navigation';
	import { apiUrl } from '$lib/api-base';
	import { playerService } from '$services/player.service';
	import { playerAdapter } from '$adapters/classes/player.adapter';
	import { mediaDetailService } from '$services/media-detail.service';
	import { libraryService } from '$services/library.service';
	import { modalRouterService } from '$services/modal-router.service';
	import Modal from '$components/core/Modal.svelte';
	import type { MediaDetailCardType } from '$types/media-detail.type';
	import TmdbLinkModal from '$components/libraries/TmdbLinkModal.svelte';
	import MediaCard from '$components/media/MediaCard.svelte';
	import MediaListCard from '$components/media/MediaListCard.svelte';
	import MediaDetail from '$components/media/MediaDetail.svelte';
	import type { MediaList, MediaListLink } from '$types/media-list.type';
	import PlayerVideo from '$components/player/PlayerVideo.svelte';
	import type { LibraryFile } from '$types/library.type';
	import type {
		MediaItem,
		MediaItemLink,
		MediaLinkSource,
		MediaCategory
	} from '$types/media-card.type';
	import type { DisplayTMDBMovieDetails, DisplayTMDBTvShowDetails } from 'tmdb/types';
	import { movieDetailsToDisplay, tvShowDetailsToDisplay } from 'tmdb/transform';

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

	// TMDB metadata state
	let tmdbMetadata: Record<string, DisplayTMDBMovieDetails | DisplayTMDBTvShowDetails> = $state({});
	let tmdbLoading: Set<string> = $state(new Set());

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
		return 'video';
	}

	function handleSelect(item: MediaItem) {
		mediaDetailService.select({
			item,
			cardType: resolveCardType(item),
			tmdbMetadata: tmdbMetadata[item.id] ?? null,
			onplay: (i) => handlePlay(i),
			onlink: (i, service) => {
				linkModalItem = i;
				linkModalService = service;
			},
			onunlink: (i, service) => handleUnlink(i, service)
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
		if (newTmdb !== sel.tmdbMetadata || updatedItem !== sel.item) {
			mediaDetailService.select({
				...sel,
				item: updatedItem,
				cardType: resolveCardType(updatedItem),
				tmdbMetadata: newTmdb
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

	async function handleListUnlink(list: MediaList, service: string) {
		const res = await fetch(apiUrl(`/api/media-lists/${list.id}/${service}`), {
			method: 'DELETE'
		});
		if (res.ok) {
			listLinkOverrides = {
				...listLinkOverrides,
				[list.id]: { ...listLinkOverrides[list.id], [service]: null }
			};
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

	$effect(() => {
		for (const item of itemsWithOverrides) {
			if (item.links.tmdb) {
				fetchTmdbMetadata(item);
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
			{/if}
			{#if selectedList.items.length > 0}
				<div
					class="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6"
				>
					{#each selectedList.items as item (item.id)}
						<MediaCard
							{item}
							tmdbMetadata={tmdbMetadata[item.id] ?? null}
							metadataLoading={tmdbLoading.has(item.id)}
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
									metadataLoading={tmdbLoading.has(item.id)}
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
					No lists yet. Scan a library with directories containing multiple video files.
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
						metadataLoading={tmdbLoading.has(item.id)}
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
