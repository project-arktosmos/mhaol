<script lang="ts">
	import type { MediaItem, CatalogResponse } from '../types';
	import { loadItems, mergeItems, clearItems } from '../storage';

	let pageTitle = $state('Loading...');
	let pageUrl = $state('');
	let error = $state('');
	let source = $state<string | null>(null);
	let instructions = $state<string | null>(null);
	let catalogItems = $state<MediaItem[]>([]);
	let newCount = $state(0);
	let loading = $state(true);

	const typeLabels: Record<string, string> = {
		movies: 'Movies',
		tv: 'TV',
		music: 'Music'
	};

	function itemFields(item: MediaItem): [string, string][] {
		const fields: [string, string][] = [['Title', item.title]];
		if (item.mediaType) fields.push(['Type', typeLabels[item.mediaType]]);
		if (item.artist) fields.push(['Artist', item.artist]);
		fields.push(['Source', item.source]);
		fields.push(['Category', item.category]);
		fields.push(['ID', item.id]);
		return fields;
	}

	async function fetchData() {
		loading = true;
		error = '';
		newCount = 0;
		try {
			// Load persisted items first
			const stored = await loadItems();
			catalogItems = stored;

			const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
			if (!tab?.id) {
				loading = false;
				return;
			}

			pageUrl = tab.url ?? '';
			pageTitle = tab.title ?? 'Unknown';

			const response: CatalogResponse = await chrome.tabs.sendMessage(tab.id, {
				type: 'GET_CATALOG'
			});
			source = response.source;
			instructions = response.instructions;

			if (response.items?.length) {
				const countBefore = catalogItems.length;
				catalogItems = await mergeItems(response.items);
				newCount = catalogItems.length - countBefore;
			}
		} catch (e) {
			// Content script may not be injected on this page — just show stored items
			if (catalogItems.length === 0) {
				error = e instanceof Error ? e.message : String(e);
			}
		}
		loading = false;
	}

	let confirmingClear = $state(false);

	async function handleClear() {
		await clearItems();
		catalogItems = [];
		newCount = 0;
		confirmingClear = false;
	}

	function handleExport() {
		const json = JSON.stringify(catalogItems, null, 2);
		const blob = new Blob([json], { type: 'application/json' });
		const url = URL.createObjectURL(blob);
		const timestamp = new Date().toISOString().slice(0, 10);
		chrome.downloads.download({ url, filename: `shepperd-catalog-${timestamp}.json`, saveAs: true });
	}

	$effect(() => {
		fetchData();
	});
</script>

<div class="bg-base-100 flex w-96 flex-col p-4" style="max-height: 560px;">
	<header class="mb-3 flex items-center justify-between">
		<div class="flex items-center gap-2">
			<div class="bg-primary h-8 w-8 rounded-lg"></div>
			<h1 class="text-lg font-bold text-white">Shepperd</h1>
		</div>
		{#if !loading}
			<div class="flex items-center gap-1">
				{#if newCount > 0}
					<span class="badge badge-success badge-sm">+{newCount} new</span>
				{/if}
				<span class="badge badge-primary badge-sm">{catalogItems.length} total</span>
			</div>
		{/if}
	</header>

	<div class="divider my-0"></div>

	{#if loading}
		<div class="flex items-center justify-center py-8">
			<span class="loading loading-spinner loading-md text-primary"></span>
		</div>
	{:else if catalogItems.length > 0}
		{#if instructions}
			<div class="bg-info/10 mt-2 rounded-lg p-2.5 text-xs leading-relaxed text-info">
				{instructions}
			</div>
		{/if}
		<div class="mt-2 flex-1 overflow-y-auto" style="max-height: 420px;">
			<div class="flex flex-col gap-2">
				{#each catalogItems as item (item.id)}
					<div class="bg-base-200 flex gap-3 rounded-lg p-2.5">
						{#if item.imageUrl}
							<img
								src={item.imageUrl}
								alt=""
								class="h-auto w-20 flex-shrink-0 self-start rounded object-cover"
							/>
						{/if}
						<table class="min-w-0 flex-1 text-xs">
							<tbody>
								{#each itemFields(item) as [key, value] (key)}
									<tr>
										<td class="whitespace-nowrap pr-2 align-top text-gray-500">{key}</td>
										<td class="break-words text-white">{value}</td>
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				{/each}
			</div>
		</div>

		<div class="mt-2 flex gap-2">
			{#if confirmingClear}
				<button class="btn btn-error btn-sm flex-1" onclick={handleClear}>Confirm delete</button>
				<button
					class="btn btn-outline btn-sm flex-1"
					onclick={() => (confirmingClear = false)}>Cancel</button
				>
			{:else}
				<button class="btn btn-primary btn-sm flex-1" onclick={fetchData}>Refresh</button>
				<button class="btn btn-outline btn-sm" onclick={handleExport}>Export</button>
				<button
					class="btn btn-outline btn-error btn-sm"
					onclick={() => (confirmingClear = true)}>Clear</button
				>
			{/if}
		</div>
	{:else if source}
		{#if instructions}
			<div class="bg-info/10 mt-2 rounded-lg p-2.5 text-xs leading-relaxed text-info">
				{instructions}
			</div>
		{/if}
		<p class="py-4 text-center text-sm text-gray-400">
			No titles found yet. Scroll the page and refresh.
		</p>
		<button class="btn btn-primary btn-sm" onclick={fetchData}>Refresh</button>
	{:else}
		<section class="mt-3">
			<span class="text-xs font-semibold uppercase tracking-wide text-gray-400">Page Title</span>
			<div class="bg-base-200 mt-1 rounded-lg p-3">
				<p class="text-sm leading-relaxed text-white">{pageTitle}</p>
			</div>
		</section>

		{#if pageUrl}
			<section class="mt-3">
				<span class="text-xs font-semibold uppercase tracking-wide text-gray-400">URL</span>
				<div class="bg-base-200 mt-1 rounded-lg p-3">
					<p class="truncate text-xs text-gray-300">{pageUrl}</p>
				</div>
			</section>
		{/if}

		<button class="btn btn-primary btn-sm mt-4" onclick={fetchData}>Refresh</button>
	{/if}

	{#if error}
		<div class="alert alert-warning mt-3 text-xs">{error}</div>
	{/if}
</div>
