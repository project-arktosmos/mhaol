<script lang="ts">
	import classNames from 'classnames';
	import { apiUrl } from '$lib/api-base';
	import TmdbLinkModal from '$components/libraries/TmdbLinkModal.svelte';
	import type { LibraryFile } from '$types/library.type';

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

	// Track link overrides so we can update without full page reload
	let linkOverrides: Record<string, Record<string, ItemLink | null>> = $state({});

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

	async function handleLink(tmdbId: number, seasonNumber: number | null, episodeNumber: number | null) {
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
		}

		linkModalItem = null;
	}

	async function handleUnlink(item: LibraryItem, service: string) {
		const res = await fetch(apiUrl(`/api/libraries/${item.libraryId}/items/${item.id}/${service}`), {
			method: 'DELETE'
		});

		if (res.ok) {
			updateItemLinks(item.id, service, null);
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
								{@const link = getItemLinks(item)[source.service]}
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
											onclick={() => { linkModalItem = item; }}
										>
											Link
										</button>
									{/if}
								</td>
							{/each}
							<td class="max-w-xs truncate text-sm opacity-70" title={item.path}>{item.path}</td>
							<td class="text-sm opacity-70">{new Date(item.createdAt).toLocaleDateString()}</td>
						</tr>
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

{#if linkModalItem}
	<TmdbLinkModal
		file={itemAsLibraryFile(linkModalItem)}
		onlink={handleLink}
		onclose={() => { linkModalItem = null; }}
	/>
{/if}
