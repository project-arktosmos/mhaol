<script lang="ts">
	import { onMount } from 'svelte';
	import classNames from 'classnames';
	import { base } from '$app/paths';
	import { goto } from '$app/navigation';
	import FirkinCard from 'ui-lib/components/firkins/FirkinCard.svelte';
	import { CONSOLE_IMAGES } from 'assets/game-consoles';
	import type { CloudFirkin } from 'ui-lib/types/firkin.type';
	import {
		listSources,
		loadGenres,
		loadPopular,
		type CatalogItem,
		type CatalogGenre,
		type CatalogSource
	} from '$lib/catalog.service';
	import { firkinsService, type Firkin } from '$lib/firkins.service';

	const firkinsStore = firkinsService.state;

	let sources = $state<CatalogSource[]>([]);
	let sourcesError = $state<string | null>(null);

	let addon = $state<string>('');
	let filter = $state<string>('');
	let page = $state<number>(1);

	let genres = $state<CatalogGenre[]>([]);
	let genresLoading = $state(false);
	let genresError = $state<string | null>(null);

	let items = $state<CatalogItem[]>([]);
	let totalPages = $state<number>(1);
	let itemsLoading = $state(false);
	let itemsError = $state<string | null>(null);

	const currentSource = $derived(sources.find((s) => s.id === addon));
	const filterLabel = $derived(currentSource?.filterLabel ?? 'Filter');
	const hasFilter = $derived(currentSource?.hasFilter ?? false);
	const isRetroAchievements = $derived(addon === 'retroachievements');
	const showFilterRow = $derived(hasFilter && !isRetroAchievements);

	const libraryFirkins = $derived<Firkin[]>(
		addon
			? $firkinsStore.firkins
					.filter((d) => d.addon === addon)
					.slice()
					.sort((a, b) => b.created_at.localeCompare(a.created_at))
					.slice(0, 6)
			: []
	);

	function virtualFirkin(item: CatalogItem): CloudFirkin {
		const images = [item.posterUrl, item.backdropUrl]
			.filter((url): url is string => Boolean(url))
			.map((url) => ({ url, mimeType: 'image/jpeg', fileSize: 0, width: 0, height: 0 }));
		return {
			id: `virtual:${addon}:${item.id}`,
			title: item.title,
			artists: [],
			description: item.description ?? '',
			images,
			files: [],
			year: item.year,
			addon,
			created_at: '',
			updated_at: '',
			version: 0,
			version_hashes: []
		};
	}

	function virtualHref(item: CatalogItem): string {
		const params = new URLSearchParams();
		params.set('addon', addon);
		params.set('id', item.id);
		params.set('title', item.title);
		if (item.year !== null && item.year !== undefined) params.set('year', String(item.year));
		if (item.description) params.set('description', item.description);
		if (item.posterUrl) params.set('posterUrl', item.posterUrl);
		if (item.backdropUrl) params.set('backdropUrl', item.backdropUrl);
		return `${base}/catalog/virtual?${params.toString()}`;
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
			genres = await loadGenres(addon);
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
			return;
		}
		itemsLoading = true;
		itemsError = null;
		try {
			const result = await loadPopular(addon, {
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
	}

	async function selectAddon(source: CatalogSource) {
		if (addon === source.id) return;
		addon = source.id;
		page = 1;
		filter = '';
		await refreshGenres();
		if (source.id === 'retroachievements') {
			items = [];
			totalPages = 1;
		} else {
			await refreshItems();
		}
	}

	async function onFilterChange() {
		page = 1;
		await refreshItems();
	}

	function selectConsole(consoleId: string) {
		void goto(`${base}/catalog/console/${encodeURIComponent(consoleId)}`);
	}

	async function goToPage(next: number) {
		if (next < 1 || next > totalPages || next === page) return;
		page = next;
		await refreshItems();
	}

	onMount(() => {
		const stopFirkins = firkinsService.start();
		void (async () => {
			try {
				sources = await listSources();
				if (sources.length > 0) {
					addon = sources[0].id;
					await refreshGenres();
					if (addon !== 'retroachievements') {
						await refreshItems();
					}
				}
			} catch (err) {
				sourcesError = err instanceof Error ? err.message : 'Unknown error';
			}
		})();
		return () => {
			stopFirkins();
		};
	});
</script>

<svelte:head>
	<title>Mhaol Cloud — Catalog</title>
</svelte:head>

<div class="flex min-h-full flex-col gap-6 p-6">
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
						<th class="w-32 align-middle">Addon</th>
						<td>
							<div class="flex flex-wrap gap-2">
								{#each sources as source (source.id)}
									{@const active = addon === source.id}
									<button
										type="button"
										class={classNames('btn btn-sm', {
											'btn-primary': active,
											'btn-outline': !active
										})}
										onclick={() => selectAddon(source)}
										title={source.kind}
									>
										{source.label}
									</button>
								{/each}
							</div>
						</td>
					</tr>
					{#if showFilterRow}
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
		<h2 class="text-lg font-semibold">Library</h2>
		{#if libraryFirkins.length === 0}
			<p class="text-sm text-base-content/60">No library items yet.</p>
		{:else}
			<div class="grid grid-cols-6 gap-4">
				{#each libraryFirkins as doc (doc.id)}
					<a
						href={`${base}/catalog/${encodeURIComponent(doc.id)}`}
						class="block no-underline"
						onclick={(e) => {
							if ((e.target as HTMLElement).closest('button, summary')) {
								e.preventDefault();
							}
						}}
					>
						<FirkinCard firkin={doc as CloudFirkin} />
					</a>
				{/each}
			</div>
		{/if}
	</section>

	{#if isRetroAchievements}
		<section class="flex flex-col gap-3">
			<h2 class="text-lg font-semibold">Consoles</h2>
			{#if genresLoading}
				<p class="text-sm text-base-content/60">Loading consoles…</p>
			{:else if genresError}
				<div class="alert alert-error">
					<span>{genresError}</span>
				</div>
			{:else if genres.length === 0}
				<p class="text-sm text-base-content/60">No consoles available.</p>
			{:else}
				<div
					class="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6"
				>
					{#each genres as console (console.id)}
						{@const consoleImage = CONSOLE_IMAGES[Number(console.id)]}
						<button
							type="button"
							class="card flex aspect-square flex-col items-center justify-center gap-2 border border-base-content/10 bg-base-200 p-4 text-center transition hover:bg-base-300"
							onclick={() => selectConsole(console.id)}
						>
							{#if consoleImage}
								<img
									src={consoleImage}
									alt={console.name}
									class="h-2/3 w-full object-contain"
									loading="lazy"
								/>
							{/if}
							<span class="text-sm font-semibold">{console.name}</span>
						</button>
					{/each}
				</div>
			{/if}
		</section>
	{:else}
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

			{#if itemsLoading && items.length === 0}
				<p class="text-sm text-base-content/60">Loading…</p>
			{:else if items.length === 0}
				<p class="text-sm text-base-content/60">No items.</p>
			{:else}
				<div
					class={classNames(
						'grid grid-cols-1 gap-4 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5',
						{ 'opacity-60': itemsLoading }
					)}
				>
					{#each items as item (item.id)}
						<a
							href={virtualHref(item)}
							class="block no-underline"
							onclick={(e) => {
								if ((e.target as HTMLElement).closest('button, summary')) {
									e.preventDefault();
								}
							}}
						>
							<FirkinCard firkin={virtualFirkin(item)} />
						</a>
					{/each}
				</div>
			{/if}
		</section>
	{/if}
</div>
