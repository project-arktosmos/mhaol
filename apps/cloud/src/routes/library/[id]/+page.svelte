<script lang="ts">
	import classNames from 'classnames';
	import { page } from '$app/stores';
	import { onMount } from 'svelte';
	import { cloudLibraryService } from 'frontend/services/cloud-library.service';
	import { cloudItemService } from 'frontend/services/cloud-item.service';
	import { cloudAdapter } from 'frontend/adapters/classes/cloud.adapter';
	import type { CloudItem } from 'frontend/types/cloud.type';

	const libStore = cloudLibraryService.store;
	const libState = cloudLibraryService.state;
	const itemState = cloudItemService.state;

	let libraryId = $derived($page.params.id ?? '');
	let libraries = $derived($libStore);
	let svc = $derived($libState);
	let iSvc = $derived($itemState);

	let library = $derived(libraries.find((l: { id: string }) => l.id === libraryId));
	let items = $derived((libraryId ? (svc.items[libraryId] ?? []) : []) as CloudItem[]);
	let loading = $derived(libraryId ? (svc.itemsLoading[libraryId] ?? false) : false);

	let filterKey = $state('');
	let filterValue = $state('');

	let filteredItems = $derived.by(() => {
		if (!filterKey || !filterValue) return items;
		return items.filter((item: CloudItem) =>
			item.attributes.some((a) => a.key === filterKey && a.value === filterValue)
		);
	});

	let distinctKeys = $derived(iSvc.distinctKeys);
	let distinctValues = $derived(filterKey ? (iSvc.distinctValues[filterKey] ?? []) : []);

	onMount(async () => {
		if (libraryId && !svc.items[libraryId]) {
			await cloudLibraryService.fetchItems(libraryId);
		}
		await cloudItemService.fetchDistinctKeys();
	});

	async function handleScan() {
		if (libraryId) await cloudLibraryService.scanLibrary(libraryId);
	}

	function handleFilterKey(key: string) {
		filterKey = key;
		filterValue = '';
		if (key) cloudItemService.fetchDistinctValues(key);
	}

	function handleFilterValue(value: string) {
		filterValue = value;
	}

	function clearFilter() {
		filterKey = '';
		filterValue = '';
	}
</script>

<div class="flex min-h-0 flex-1">
	<aside class="w-64 shrink-0 overflow-y-auto bg-base-200 p-4">
		<h3 class="mb-3 text-sm font-semibold tracking-wide uppercase">Filter by Attribute</h3>

		{#if filterKey}
			<button class="btn mb-2 btn-ghost btn-xs" onclick={clearFilter}>Clear filter</button>
		{/if}

		<div class="space-y-1">
			{#each distinctKeys as key (key)}
				<button
					class={classNames('btn w-full justify-start btn-ghost btn-sm', {
						'btn-active': filterKey === key
					})}
					onclick={() => handleFilterKey(key)}
				>
					{key}
				</button>
			{/each}
		</div>

		{#if filterKey && distinctValues.length > 0}
			<div class="divider"></div>
			<h4 class="mb-2 text-xs font-semibold uppercase">{filterKey} values</h4>
			<div class="space-y-1">
				{#each distinctValues as value (value)}
					<button
						class={classNames('btn w-full justify-start btn-ghost btn-xs', {
							'btn-active': filterValue === value
						})}
						onclick={() => handleFilterValue(value)}
					>
						{value}
					</button>
				{/each}
			</div>
		{/if}
	</aside>

	<div class="flex-1 overflow-y-auto p-6">
		<div class="mb-4 flex items-center justify-between">
			<div>
				<h1 class="text-2xl font-bold">{library?.name ?? 'Library'}</h1>
				<p class="text-sm text-base-content/60">
					{filteredItems.length} items
					{#if filterKey}&middot; filtered by {filterKey}{filterValue
							? ` = ${filterValue}`
							: ''}{/if}
				</p>
			</div>
			<div class="flex gap-2">
				<a href="/" class="btn btn-ghost btn-sm">Back</a>
				<button class="btn btn-sm btn-secondary" disabled={loading} onclick={handleScan}>
					{loading ? 'Scanning...' : 'Scan'}
				</button>
			</div>
		</div>

		<div class="overflow-x-auto">
			<table class="table table-sm">
				<thead>
					<tr>
						<th>Filename</th>
						<th>Extension</th>
						<th>Size</th>
						<th>Type</th>
						<th>Attributes</th>
						<th></th>
					</tr>
				</thead>
				<tbody>
					{#each filteredItems as item (item.id)}
						<tr class="hover">
							<td class="max-w-xs truncate font-medium">{item.filename}</td>
							<td>
								<span
									class={classNames(
										'badge badge-sm',
										cloudAdapter.extensionBadgeClass(item.extension)
									)}
								>
									{item.extension}
								</span>
							</td>
							<td class="text-xs text-base-content/60">
								{cloudAdapter.formatBytes(item.sizeBytes)}
							</td>
							<td class="text-xs text-base-content/60">{item.mimeType ?? '-'}</td>
							<td>
								<div class="flex flex-wrap gap-1">
									{#each item.attributes.slice(0, 3) as attr (attr.key + attr.source)}
										<span class="badge badge-ghost badge-xs">{attr.key}: {attr.value}</span>
									{/each}
									{#if item.attributes.length > 3}
										<span class="badge badge-ghost badge-xs">+{item.attributes.length - 3}</span>
									{/if}
								</div>
							</td>
							<td>
								<a href="/item/{item.id}" class="btn btn-ghost btn-xs">Detail</a>
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>

		{#if filteredItems.length === 0 && !loading}
			<div class="py-12 text-center">
				<p class="text-base-content/60">
					{items.length === 0
						? 'No items. Try scanning the library.'
						: 'No items match the filter.'}
				</p>
			</div>
		{/if}
	</div>
</div>
