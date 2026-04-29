<script lang="ts">
	import { onMount } from 'svelte';
	import classNames from 'classnames';
	import {
		listSources,
		loadGenres,
		loadPopular,
		type CatalogItem,
		type CatalogGenre,
		type CatalogSource
	} from '$lib/catalog.service';
	import {
		formatSizeBytes,
		matchTorrentsForResult,
		searchTorrents,
		type SearchResultItem,
		type TorrentResultItem
	} from '$lib/search.service';
	import { documentsService, type DocumentSource, type DocumentType } from '$lib/documents.service';
	import { cachedImageUrl } from '$lib/image-cache';

	let sources = $state<CatalogSource[]>([]);
	let sourcesError = $state<string | null>(null);

	let addon = $state<string>('');
	let type = $state<string>('');
	let filter = $state<string>('');
	let page = $state<number>(1);

	let genres = $state<CatalogGenre[]>([]);
	let genresLoading = $state(false);
	let genresError = $state<string | null>(null);

	let items = $state<CatalogItem[]>([]);
	let totalPages = $state<number>(1);
	let itemsLoading = $state(false);
	let itemsError = $state<string | null>(null);

	type ItemTorrentStatus = 'pending' | 'searching' | 'done' | 'error';
	interface ItemTorrentState {
		status: ItemTorrentStatus;
		matches: TorrentResultItem[];
		error: string | null;
	}
	let torrentState = $state<Record<string, ItemTorrentState>>({});
	let addedHashes = $state<Set<string>>(new Set());
	let addingHash = $state<string | null>(null);
	let assignError = $state<string | null>(null);
	let runId = 0;

	const currentSource = $derived(sources.find((s) => s.id === addon));
	const availableTypes = $derived(currentSource?.types ?? []);
	const filterLabel = $derived(currentSource?.filterLabel ?? 'Filter');
	const hasFilter = $derived(currentSource?.hasFilter ?? false);

	$effect(() => {
		if (currentSource && availableTypes.length > 0 && !availableTypes.some((t) => t.id === type)) {
			type = availableTypes[0]?.id ?? '';
		}
	});

	function mapToDocumentType(addonId: string, typeId: string): DocumentType {
		if (addonId === 'tmdb' && typeId === 'tv') return 'tv show';
		if (addonId === 'tmdb') return 'movie';
		if (addonId === 'musicbrainz') return 'album';
		if (addonId === 'openlibrary') return 'book';
		if (addonId === 'retroachievements') return 'game';
		return 'movie';
	}

	function mapToDocumentSource(addonId: string): DocumentSource {
		if (addonId === 'tmdb') return 'tmdb';
		if (addonId === 'musicbrainz') return 'musicbrainz';
		if (addonId === 'openlibrary') return 'openlibrary';
		if (addonId === 'retroachievements') return 'retroachievements';
		return 'tmdb';
	}

	function itemAsSearchResult(item: CatalogItem): SearchResultItem {
		return {
			title: item.title,
			description: item.description ?? '',
			artists: [],
			images: [],
			files: [],
			year: item.year,
			externalId: item.id,
			raw: item
		};
	}

	async function refreshGenres() {
		if (!addon || !hasFilter) {
			genres = [];
			filter = '';
			return;
		}
		genresLoading = true;
		genresError = null;
		try {
			genres = await loadGenres(addon, type || undefined);
			if (!genres.some((g) => g.id === filter)) {
				filter = genres[0]?.id ?? '';
			}
		} catch (err) {
			genres = [];
			genresError = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			genresLoading = false;
		}
	}

	async function refreshItems() {
		if (!addon) {
			items = [];
			torrentState = {};
			return;
		}
		itemsLoading = true;
		itemsError = null;
		runId++;
		torrentState = {};
		try {
			const result = await loadPopular(addon, {
				type: type || undefined,
				filter: filter || undefined,
				page
			});
			items = result.items;
			totalPages = result.totalPages;
			page = result.page;
		} catch (err) {
			items = [];
			totalPages = 1;
			itemsError = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			itemsLoading = false;
		}
		void runTorrentSearches();
	}

	async function runTorrentSearches() {
		const myRun = ++runId;
		const docType = mapToDocumentType(addon, type);
		const seeded: Record<string, ItemTorrentState> = {};
		for (const item of items) {
			seeded[item.id] = { status: 'pending', matches: [], error: null };
		}
		torrentState = seeded;
		for (const item of items) {
			if (myRun !== runId) return;
			torrentState = {
				...torrentState,
				[item.id]: { status: 'searching', matches: [], error: null }
			};
			try {
				const torrents = await searchTorrents(docType, item.title);
				if (myRun !== runId) return;
				const matches = matchTorrentsForResult(itemAsSearchResult(item), torrents);
				torrentState = {
					...torrentState,
					[item.id]: { status: 'done', matches, error: null }
				};
			} catch (err) {
				if (myRun !== runId) return;
				torrentState = {
					...torrentState,
					[item.id]: {
						status: 'error',
						matches: [],
						error: err instanceof Error ? err.message : 'Unknown error'
					}
				};
			}
		}
	}

	async function assignTorrent(item: CatalogItem, torrent: TorrentResultItem) {
		if (!torrent.magnetLink || addedHashes.has(torrent.magnetLink) || addingHash) return;
		assignError = null;
		addingHash = torrent.magnetLink;
		try {
			const images = [item.posterUrl, item.backdropUrl]
				.filter((url): url is string => Boolean(url))
				.map((url) => ({ url, mimeType: 'image/jpeg', fileSize: 0, width: 0, height: 0 }));
			await documentsService.create({
				title: item.title,
				artists: [],
				description: item.description ?? '',
				images,
				files: [{ type: 'torrent magnet', value: torrent.magnetLink, title: torrent.title }],
				year: item.year,
				type: mapToDocumentType(addon, type),
				source: mapToDocumentSource(addon)
			});
			addedHashes = new Set(addedHashes).add(torrent.magnetLink);
		} catch (err) {
			assignError = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			addingHash = null;
		}
	}

	async function onAddonChange() {
		page = 1;
		filter = '';
		await refreshGenres();
		await refreshItems();
	}

	async function onTypeChange() {
		page = 1;
		filter = '';
		await refreshGenres();
		await refreshItems();
	}

	async function onFilterChange() {
		page = 1;
		await refreshItems();
	}

	async function goToPage(next: number) {
		if (next < 1 || next > totalPages || next === page) return;
		page = next;
		await refreshItems();
	}

	onMount(async () => {
		try {
			sources = await listSources();
			if (sources.length > 0) {
				addon = sources[0].id;
				type = sources[0].types[0]?.id ?? '';
				await refreshGenres();
				await refreshItems();
			}
		} catch (err) {
			sourcesError = err instanceof Error ? err.message : 'Unknown error';
		}
	});
</script>

<svelte:head>
	<title>Mhaol Cloud — Catalog</title>
</svelte:head>

<div class="flex min-h-full flex-col gap-6 p-6">
	<header class="flex flex-col gap-1">
		<h1 class="text-2xl font-bold">Catalog</h1>
		<p class="text-sm text-base-content/60">
			Browse popular items from each addon. Pick a source, optionally narrow by type and genre, and
			the cloud server fetches the data on your behalf. For each result the cloud also runs a
			torrent search (one at a time) and lists the matches; click a torrent to save the item as a
			document with that magnet attached.
		</p>
	</header>

	{#if sourcesError}
		<div class="alert alert-error">
			<span>Could not load catalog sources: {sourcesError}</span>
		</div>
	{/if}

	<section class="card border border-base-content/10 bg-base-200 p-4">
		<div class="overflow-x-auto rounded-box border border-base-content/10">
			<table class="table table-sm">
				<tbody>
					<tr>
						<th class="w-32 align-middle">Source</th>
						<td>
							<select
								class="select-bordered select w-full select-sm"
								bind:value={addon}
								onchange={onAddonChange}
								disabled={sources.length === 0}
							>
								{#each sources as option (option.id)}
									<option value={option.id}>{option.label}</option>
								{/each}
							</select>
						</td>
					</tr>
					{#if availableTypes.length > 1}
						<tr>
							<th class="w-32 align-middle">Type</th>
							<td>
								<select
									class="select-bordered select w-full select-sm"
									bind:value={type}
									onchange={onTypeChange}
								>
									{#each availableTypes as option (option.id)}
										<option value={option.id}>{option.label}</option>
									{/each}
								</select>
							</td>
						</tr>
					{/if}
					{#if hasFilter}
						<tr>
							<th class="w-32 align-middle">{filterLabel}</th>
							<td>
								{#if genresLoading}
									<span class="text-xs text-base-content/60"
										>Loading {filterLabel.toLowerCase()}…</span
									>
								{:else if genresError}
									<span class="text-xs text-error">{genresError}</span>
								{:else if genres.length === 0}
									<span class="text-xs text-base-content/60">No options available</span>
								{:else}
									<select
										class="select-bordered select w-full select-sm"
										bind:value={filter}
										onchange={onFilterChange}
									>
										{#each genres as option (option.id)}
											<option value={option.id}>{option.name}</option>
										{/each}
									</select>
								{/if}
							</td>
						</tr>
					{/if}
				</tbody>
			</table>
		</div>
	</section>

	<section class="flex flex-col gap-3">
		<div class="flex items-center justify-between gap-4">
			<h2 class="text-lg font-semibold">Popular</h2>
			<div class="flex items-center gap-2">
				<button
					class="btn btn-outline btn-xs"
					onclick={() => goToPage(page - 1)}
					disabled={itemsLoading || page <= 1}
				>
					Prev
				</button>
				<span class="text-xs text-base-content/60">Page {page} / {totalPages}</span>
				<button
					class="btn btn-outline btn-xs"
					onclick={() => goToPage(page + 1)}
					disabled={itemsLoading || page >= totalPages}
				>
					Next
				</button>
				<button class="btn btn-outline btn-xs" onclick={refreshItems} disabled={itemsLoading}>
					Refresh
				</button>
			</div>
		</div>

		{#if itemsError}
			<div class="alert alert-error">
				<span>{itemsError}</span>
			</div>
		{/if}

		{#if assignError}
			<div class="alert alert-error">
				<span>{assignError}</span>
			</div>
		{/if}

		{#if itemsLoading && items.length === 0}
			<p class="text-sm text-base-content/60">Loading…</p>
		{:else if items.length === 0}
			<p class="text-sm text-base-content/60">No items.</p>
		{:else}
			<div
				class={classNames('grid grid-cols-1 gap-3 md:grid-cols-2 lg:grid-cols-3', {
					'opacity-60': itemsLoading
				})}
			>
				{#each items as item (item.id)}
					{@const ts = torrentState[item.id]}
					<div class="flex overflow-hidden rounded-box border border-base-content/10 bg-base-100">
						<div class="flex w-32 shrink-0 flex-col bg-base-300">
							<div class="aspect-[2/3] w-full">
								{#if item.posterUrl}
									<img
										src={cachedImageUrl(item.posterUrl)}
										alt={item.title}
										class="h-full w-full object-cover"
										loading="lazy"
									/>
								{:else}
									<div
										class="flex h-full w-full items-center justify-center text-xs text-base-content/40"
									>
										No image
									</div>
								{/if}
							</div>
							<div class="flex flex-col gap-1 p-2">
								<span class="line-clamp-2 text-sm font-medium" title={item.title}>{item.title}</span
								>
								{#if item.year}
									<span class="text-xs text-base-content/60">{item.year}</span>
								{/if}
								{#if item.description}
									<span class="line-clamp-3 text-xs text-base-content/70">{item.description}</span>
								{/if}
							</div>
						</div>
						<div class="flex flex-1 flex-col border-l border-base-content/10 p-2">
							<span class="mb-1 text-xs font-semibold text-base-content/60 uppercase">
								Torrents{ts && ts.matches.length > 0 ? ` (${ts.matches.length})` : ''}
							</span>
							{#if !ts || ts.status === 'pending'}
								<p class="text-xs text-base-content/50">Queued…</p>
							{:else if ts.status === 'searching'}
								<p class="text-xs text-base-content/50">Searching…</p>
							{:else if ts.status === 'error'}
								<p class="text-xs text-error">{ts.error ?? 'Failed'}</p>
							{:else if ts.matches.length === 0}
								<p class="text-xs text-base-content/50">No matching torrents.</p>
							{:else}
								<div class="flex max-h-48 flex-col gap-1 overflow-y-auto">
									{#each ts.matches as torrent (torrent.infoHash)}
										<button
											type="button"
											class={classNames(
												'flex flex-wrap items-center gap-2 rounded border border-base-content/10 px-2 py-1 text-left text-xs hover:bg-base-200',
												{
													'opacity-60':
														addedHashes.has(torrent.magnetLink) || addingHash === torrent.magnetLink
												}
											)}
											onclick={() => assignTorrent(item, torrent)}
											disabled={addingHash !== null || addedHashes.has(torrent.magnetLink)}
											title={torrent.title}
										>
											<span class="font-medium">{torrent.quality ?? '—'}</span>
											<span class="text-success">↑{torrent.seeders}</span>
											<span class="text-warning">↓{torrent.leechers}</span>
											<span class="text-base-content/60">{formatSizeBytes(torrent.sizeBytes)}</span>
											{#if addedHashes.has(torrent.magnetLink)}
												<span class="ml-auto">✓</span>
											{:else if addingHash === torrent.magnetLink}
												<span class="ml-auto">…</span>
											{/if}
										</button>
									{/each}
								</div>
							{/if}
						</div>
					</div>
				{/each}
			</div>
		{/if}
	</section>
</div>
