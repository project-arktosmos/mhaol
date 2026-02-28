<script lang="ts">
	import classNames from 'classnames';
	import { apiUrl } from '$lib/api-base';
	import TmdbLinkModal from '$components/libraries/TmdbLinkModal.svelte';
	import MusicBrainzLinkModal from '$components/libraries/MusicBrainzLinkModal.svelte';
	import type { LibraryFile } from '$types/library.type';
	import type { DisplayTMDBMovieDetails, DisplayTMDBTvShowDetails } from 'tmdb/types';
	import { movieDetailsToDisplay, tvShowDetailsToDisplay } from 'tmdb/transform';

	interface ItemLink {
		serviceId: string;
		seasonNumber: number | null;
		episodeNumber: number | null;
	}

	interface LibraryItem {
		id: string;
		libraryId: string;
		name: string;
		extension: string;
		path: string;
		categoryId: string | null;
		createdAt: string;
		links: Record<string, ItemLink>;
	}

	interface LinkSource {
		id: string;
		service: string;
		label: string;
		mediaTypeId: string;
		categoryId: string | null;
	}

	interface Category {
		id: string;
		mediaTypeId: string;
		label: string;
	}

	interface Props {
		data: {
			mediaTypes: Array<{ id: string; label: string }>;
			categories: Category[];
			linkSources: LinkSource[];
			itemsByCategory: Record<string, LibraryItem[]>;
			itemsByType: Record<string, LibraryItem[]>;
		};
	}

	const ALL = '__all__';

	let { data }: Props = $props();

	let activeTypeId = $state('');
	let activeCategoryId = $state(ALL);
	let linkModalItem: LibraryItem | null = $state(null);
	let linkModalService: string | null = $state(null);

	// Track link overrides so we can update without full page reload
	let linkOverrides: Record<string, Record<string, ItemLink | null>> = $state({});

	// TMDB metadata state
	let tmdbMetadata: Record<string, DisplayTMDBMovieDetails | DisplayTMDBTvShowDetails> = $state({});
	let tmdbLoading: Set<string> = $state(new Set());

	function getItemLinks(item: LibraryItem): Record<string, ItemLink> {
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

	let activeType = $derived(activeTypeId || data.mediaTypes[0]?.id || '');

	let categoriesForType = $derived(
		data.categories.filter((c) => c.mediaTypeId === activeType)
	);

	let activeCategory = $derived.by(() => {
		if (activeCategoryId === ALL) return ALL;
		if (categoriesForType.some((c) => c.id === activeCategoryId)) return activeCategoryId;
		return ALL;
	});

	let isAllView = $derived(activeCategory === ALL);

	let items = $derived(
		isAllView
			? data.itemsByType[activeType] ?? []
			: data.itemsByCategory[activeCategory] ?? []
	);

	let categoryLabelMap = $derived(
		Object.fromEntries(data.categories.map((c) => [c.id, c.label]))
	);

	let activeLinkSources = $derived.by(() => {
		const seen = new Set<string>();
		const sources: LinkSource[] = [];

		for (const ls of data.linkSources) {
			if (ls.mediaTypeId !== activeType) continue;
			if (seen.has(ls.service)) continue;

			if (isAllView) {
				if (ls.categoryId === null) {
					seen.add(ls.service);
					sources.push(ls);
				} else {
					// In "All" view, show sources registered for any category (deduplicated)
					seen.add(ls.service);
					sources.push(ls);
				}
			} else {
				if (ls.categoryId === null || ls.categoryId === activeCategory) {
					seen.add(ls.service);
					sources.push(ls);
				}
			}
		}

		return sources;
	});

	function selectType(id: string) {
		activeTypeId = id;
		activeCategoryId = ALL;
	}

	function selectCategory(id: string) {
		activeCategoryId = id;
	}

	function updateItemLinks(itemId: string, service: string, link: ItemLink | null) {
		if (!linkOverrides[itemId]) {
			linkOverrides[itemId] = {};
		}
		linkOverrides[itemId][service] = link;
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
			updateItemLinks(item.id, 'tmdb', {
				serviceId: String(tmdbId),
				seasonNumber,
				episodeNumber
			});

			const categoryId = type === 'movie' ? 'movies' : 'tv';
			if (item.categoryId !== categoryId) {
				await fetch(apiUrl(`/api/libraries/${item.libraryId}/items/${item.id}/category`), {
					method: 'PUT',
					headers: { 'Content-Type': 'application/json' },
					body: JSON.stringify({ categoryId })
				});
				item.categoryId = categoryId;
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

	async function handleUnlink(item: LibraryItem, service: string) {
		const res = await fetch(apiUrl(`/api/libraries/${item.libraryId}/items/${item.id}/${service}`), {
			method: 'DELETE'
		});

		if (res.ok) {
			updateItemLinks(item.id, service, null);
			if (service === 'tmdb') {
				delete tmdbMetadata[item.id];
			}
		}
	}

	function itemAsLibraryFile(item: LibraryItem): LibraryFile {
		return {
			id: item.id,
			name: item.name,
			path: item.path,
			extension: item.extension,
			mediaType: activeType as LibraryFile['mediaType'],
			categoryId: item.categoryId,
			links: getItemLinks(item)
		};
	}

	async function fetchTmdbMetadata(item: LibraryItem) {
		const links = getItemLinks(item);
		const tmdbLink = links.tmdb;
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

	$effect(() => {
		for (const item of items) {
			const links = getItemLinks(item);
			if (links.tmdb) {
				fetchTmdbMetadata(item);
			}
		}
	});

	function isTmdbMovieDetails(
		meta: DisplayTMDBMovieDetails | DisplayTMDBTvShowDetails
	): meta is DisplayTMDBMovieDetails {
		return 'title' in meta;
	}
</script>

<div class="container mx-auto p-4">
	<div class="mb-6">
		<h1 class="text-3xl font-bold">Media</h1>
		<p class="text-sm opacity-70">Browse your media library</p>
	</div>

	<!-- Tier 1: Media Types -->
	<div class="mb-3 flex flex-wrap gap-2">
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
					'btn-secondary': isAllView,
					'btn-ghost': !isAllView
				})}
				onclick={() => selectCategory(ALL)}
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

	<!-- Items table -->
	{#if items.length > 0}
		<div class="overflow-x-auto">
			<table class="table table-zebra w-full">
				<thead>
					<tr>
						<th>Name</th>
						<th>Extension</th>
						{#if isAllView}
							<th>Category</th>
						{/if}
						{#each activeLinkSources as source (source.service)}
							<th>{source.label}</th>
						{/each}
						<th>Path</th>
						<th>Added</th>
					</tr>
				</thead>
				<tbody>
					{#each items as item (item.id)}
						{@const itemLinks = getItemLinks(item)}
						<tr>
							<td class="font-medium">{item.name}</td>
							<td><span class="badge badge-ghost badge-sm">{item.extension}</span></td>
							{#if isAllView}
								<td>
									<span class="badge badge-outline badge-sm">
										{item.categoryId ? (categoryLabelMap[item.categoryId] ?? item.categoryId) : 'Uncategorized'}
									</span>
								</td>
							{/if}
							{#each activeLinkSources as source (source.service)}
								{@const link = itemLinks[source.service]}
								<td>
									{#if link}
										<span class="badge badge-info badge-sm gap-1">
											{link.serviceId}
											<button
												class="cursor-pointer opacity-60 hover:opacity-100"
												onclick={() => handleUnlink(item, source.service)}
												title="Unlink"
											>&times;</button>
										</span>
									{:else}
										<button
											class="btn btn-ghost btn-xs"
											onclick={() => { linkModalItem = item; linkModalService = source.service; }}
										>
											Link
										</button>
									{/if}
								</td>
							{/each}
							<td class="max-w-xs truncate text-sm opacity-70" title={item.path}>{item.path}</td>
							<td class="text-sm opacity-70">{new Date(item.createdAt).toLocaleDateString()}</td>
						</tr>
						{#if itemLinks.tmdb}
							{@const colSpan = 4 + activeLinkSources.length + (isAllView ? 1 : 0)}
							<tr>
								<td colspan={colSpan} class="bg-base-200 p-0">
									{#if tmdbLoading.has(item.id)}
										<div class="flex justify-center py-6">
											<span class="loading loading-spinner loading-md"></span>
										</div>
									{:else if tmdbMetadata[item.id]}
										{@const meta = tmdbMetadata[item.id]}
										<div class="flex gap-4 p-4">
											{#if isTmdbMovieDetails(meta)}
												{#if meta.posterUrl}
													<img
														src={meta.posterUrl}
														alt={meta.title}
														class="h-36 w-24 flex-shrink-0 rounded object-cover"
													/>
												{/if}
												<div class="min-w-0 flex-1">
													<div class="flex flex-wrap items-center gap-2">
														<h3 class="text-lg font-bold">{meta.title}</h3>
														<span class="text-sm opacity-70">{meta.releaseYear}</span>
														{#if meta.runtime}
															<span class="badge badge-outline badge-sm">{meta.runtime}</span>
														{/if}
														{#if meta.voteAverage > 0}
															<span class="flex items-center gap-1">
																<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" class="h-4 w-4 text-yellow-500">
																	<path fill-rule="evenodd" d="M10.788 3.21c.448-1.077 1.976-1.077 2.424 0l2.082 5.007 5.404.433c1.164.093 1.636 1.545.749 2.305l-4.117 3.527 1.257 5.273c.271 1.136-.964 2.033-1.96 1.425L12 18.354 7.373 21.18c-.996.608-2.231-.29-1.96-1.425l1.257-5.273-4.117-3.527c-.887-.76-.415-2.212.749-2.305l5.404-.433 2.082-5.006z" clip-rule="evenodd" />
																</svg>
																<span class="text-sm font-semibold">{meta.voteAverage.toFixed(1)}</span>
															</span>
														{/if}
													</div>
													{#if meta.genres.length > 0}
														<div class="mt-1 flex flex-wrap gap-1">
															{#each meta.genres as genre}
																<span class="badge badge-primary badge-outline badge-xs">{genre}</span>
															{/each}
														</div>
													{/if}
													{#if meta.director}
														<div class="mt-2 text-sm">
															<span class="font-semibold">Director:</span> {meta.director}
														</div>
													{/if}
													{#if meta.overview}
														<p class="mt-2 line-clamp-3 text-sm opacity-80">{meta.overview}</p>
													{/if}
												</div>
											{:else}
												{#if meta.posterUrl}
													<img
														src={meta.posterUrl}
														alt={meta.name}
														class="h-36 w-24 flex-shrink-0 rounded object-cover"
													/>
												{/if}
												<div class="min-w-0 flex-1">
													<div class="flex flex-wrap items-center gap-2">
														<h3 class="text-lg font-bold">{meta.name}</h3>
														<span class="text-sm opacity-70">
															{meta.firstAirYear}{meta.lastAirYear ? ` - ${meta.lastAirYear}` : ''}
														</span>
														{#if meta.status}
															<span class="badge badge-outline badge-sm">{meta.status}</span>
														{/if}
														{#if meta.voteAverage > 0}
															<span class="flex items-center gap-1">
																<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" class="h-4 w-4 text-yellow-500">
																	<path fill-rule="evenodd" d="M10.788 3.21c.448-1.077 1.976-1.077 2.424 0l2.082 5.007 5.404.433c1.164.093 1.636 1.545.749 2.305l-4.117 3.527 1.257 5.273c.271 1.136-.964 2.033-1.96 1.425L12 18.354 7.373 21.18c-.996.608-2.231-.29-1.96-1.425l1.257-5.273-4.117-3.527c-.887-.76-.415-2.212.749-2.305l5.404-.433 2.082-5.006z" clip-rule="evenodd" />
																</svg>
																<span class="text-sm font-semibold">{meta.voteAverage.toFixed(1)}</span>
															</span>
														{/if}
													</div>
													{#if meta.genres.length > 0}
														<div class="mt-1 flex flex-wrap gap-1">
															{#each meta.genres as genre}
																<span class="badge badge-primary badge-outline badge-xs">{genre}</span>
															{/each}
														</div>
													{/if}
													{#if meta.numberOfSeasons || meta.numberOfEpisodes}
														<div class="mt-2 flex gap-3 text-sm opacity-70">
															{#if meta.numberOfSeasons}
																<span>{meta.numberOfSeasons} season{meta.numberOfSeasons !== 1 ? 's' : ''}</span>
															{/if}
															{#if meta.numberOfEpisodes}
																<span>{meta.numberOfEpisodes} episode{meta.numberOfEpisodes !== 1 ? 's' : ''}</span>
															{/if}
														</div>
													{/if}
													{#if meta.createdBy.length > 0}
														<div class="mt-2 text-sm">
															<span class="font-semibold">Created by:</span> {meta.createdBy.join(', ')}
														</div>
													{/if}
													{#if meta.overview}
														<p class="mt-2 line-clamp-3 text-sm opacity-80">{meta.overview}</p>
													{/if}
												</div>
											{/if}
										</div>
									{/if}
								</td>
							</tr>
						{/if}
					{/each}
				</tbody>
			</table>
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
