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

	const currentSource = $derived(sources.find((s) => s.id === addon));
	const availableTypes = $derived(currentSource?.types ?? []);
	const filterLabel = $derived(currentSource?.filterLabel ?? 'Filter');
	const hasFilter = $derived(currentSource?.hasFilter ?? false);

	$effect(() => {
		if (currentSource && availableTypes.length > 0 && !availableTypes.some((t) => t.id === type)) {
			type = availableTypes[0]?.id ?? '';
		}
	});

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
			return;
		}
		itemsLoading = true;
		itemsError = null;
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
			the cloud server fetches the data on your behalf.
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

		{#if itemsLoading && items.length === 0}
			<p class="text-sm text-base-content/60">Loading…</p>
		{:else if items.length === 0}
			<p class="text-sm text-base-content/60">No items.</p>
		{:else}
			<div
				class={classNames(
					'grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6',
					{ 'opacity-60': itemsLoading }
				)}
			>
				{#each items as item (item.id)}
					<div
						class="flex flex-col overflow-hidden rounded-box border border-base-content/10 bg-base-100"
					>
						<div class="aspect-[2/3] w-full bg-base-300">
							{#if item.posterUrl}
								<img
									src={item.posterUrl}
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
						<div class="flex flex-1 flex-col gap-1 p-2">
							<span class="line-clamp-2 text-sm font-medium" title={item.title}>{item.title}</span>
							{#if item.year}
								<span class="text-xs text-base-content/60">{item.year}</span>
							{/if}
							{#if item.description}
								<span class="line-clamp-3 text-xs text-base-content/70">{item.description}</span>
							{/if}
						</div>
					</div>
				{/each}
			</div>
		{/if}
	</section>
</div>
