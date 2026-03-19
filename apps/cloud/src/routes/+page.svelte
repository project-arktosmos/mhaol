<script lang="ts">
	import classNames from 'classnames';
	import { onMount } from 'svelte';
	import { cloudLibraryService } from 'frontend/services/cloud-library.service';
	import { cloudPeerService } from 'frontend/services/cloud-peer.service';
	import { cloudItemService } from 'frontend/services/cloud-item.service';
	import { cloudAdapter } from 'frontend/adapters/classes/cloud.adapter';
	import type { CloudItem } from 'frontend/types/cloud.type';

	const libStore = cloudLibraryService.store;
	const libState = cloudLibraryService.state;
	const peerState = cloudPeerService.state;
	const itemState = cloudItemService.state;

	let libraries = $derived($libStore);
	let svc = $derived($libState);
	let peers = $derived($peerState);
	let iSvc = $derived($itemState);

	let peerLibraries = $derived(
		Object.entries(peers.peers).flatMap(([peerId, data]) =>
			data.libraries.map((lib) => ({ ...lib, peerId }))
		)
	);

	let libraryNameMap = $derived(Object.fromEntries(libraries.map((l) => [l.id, l.name])));

	let allItems = $derived(Object.values(svc.items).flat() as CloudItem[]);

	let anyLoading = $derived(Object.values(svc.itemsLoading).some(Boolean));

	let filterLibraryId = $state('');
	let filterKey = $state('');
	let filterValue = $state('');

	let filteredItems = $derived.by(() => {
		let result = allItems;
		if (filterLibraryId) {
			result = result.filter((item) => item.libraryId === filterLibraryId);
		}
		if (filterKey && filterValue) {
			result = result.filter((item: CloudItem) =>
				item.attributes.some((a) => a.key === filterKey && a.value === filterValue)
			);
		}
		return result;
	});

	let distinctKeys = $derived(iSvc.distinctKeys);
	let distinctValues = $derived(filterKey ? (iSvc.distinctValues[filterKey] ?? []) : []);

	// Detail panel
	let selectedItemId = $state<string | null>(null);
	let detailItem = $derived(iSvc.currentItem);
	let detailLoading = $derived(iSvc.loading);

	let newAttrKey = $state('');
	let newAttrValue = $state('');
	let newAttrType = $state('string');

	onMount(async () => {
		await cloudItemService.fetchDistinctKeys();
	});

	function handleSelectItem(itemId: string) {
		selectedItemId = itemId;
		cloudItemService.getItem(itemId);
	}

	function handleCloseDetail() {
		selectedItemId = null;
	}

	async function handleAddAttribute() {
		if (!newAttrKey || !newAttrValue || !selectedItemId) return;
		await cloudItemService.setAttribute(selectedItemId, newAttrKey, newAttrValue, newAttrType);
		newAttrKey = '';
		newAttrValue = '';
		newAttrType = 'string';
	}

	async function handleRemoveAttribute(key: string) {
		if (selectedItemId) await cloudItemService.removeAttribute(selectedItemId, key);
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
		filterLibraryId = '';
	}
</script>

<div class="flex min-h-0 flex-1">
	<aside class="w-64 shrink-0 overflow-y-auto bg-base-200 p-4">
		<h3 class="mb-3 text-sm font-semibold tracking-wide uppercase">Libraries</h3>

		<div class="space-y-1">
			<button
				class={classNames('btn w-full justify-start btn-ghost btn-sm', {
					'btn-active': filterLibraryId === ''
				})}
				onclick={() => (filterLibraryId = '')}
			>
				All ({allItems.length})
			</button>
			{#each libraries as lib (lib.id)}
				<button
					class={classNames('btn w-full justify-between btn-ghost btn-sm', {
						'btn-active': filterLibraryId === lib.id
					})}
					onclick={() => (filterLibraryId = lib.id)}
				>
					<span class="truncate">{lib.name}</span>
					<span class="ml-1 badge badge-xs badge-info">Local</span>
				</button>
			{/each}
			{#each peerLibraries as peerLib (peerLib.peerId + '-' + peerLib.id)}
				<div class="flex items-center gap-1 px-2 py-1">
					<span class="truncate text-xs text-base-content/50">{peerLib.name}</span>
					<span class="ml-auto badge badge-xs badge-warning">Peer</span>
				</div>
			{/each}
		</div>

		<div class="divider"></div>

		<h3 class="mb-3 text-sm font-semibold tracking-wide uppercase">Filter by Attribute</h3>

		{#if filterKey || filterLibraryId}
			<button class="btn mb-2 btn-ghost btn-xs" onclick={clearFilter}>Clear filters</button>
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
		<div class="overflow-x-auto">
			<table class="table table-sm">
				<thead>
					<tr>
						<th>Filename</th>
						<th>Library</th>
						<th>Extension</th>
						<th>Size</th>
						<th>Type</th>
						<th>Attributes</th>
					</tr>
				</thead>
				<tbody>
					{#each filteredItems as item (item.id)}
						<tr
							class={classNames('cursor-pointer', {
								'bg-base-200': selectedItemId === item.id,
								hover: selectedItemId !== item.id
							})}
							onclick={() => handleSelectItem(item.id)}
						>
							<td class="max-w-xs truncate font-medium">{item.filename}</td>
							<td class="text-xs text-base-content/60">
								{libraryNameMap[item.libraryId] ?? 'Unknown'}
							</td>
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
						</tr>
					{/each}
				</tbody>
			</table>
		</div>

		{#if filteredItems.length === 0 && !anyLoading}
			<div class="py-12 text-center">
				<p class="text-base-content/60">
					{#if allItems.length === 0}
						No items yet. Add a library and scan it to get started.
					{:else}
						No items match the current filters.
					{/if}
				</p>
			</div>
		{/if}
	</div>

	{#if selectedItemId}
		<aside class="w-96 shrink-0 overflow-y-auto border-l border-base-300 bg-base-100 p-4">
			{#if detailLoading}
				<div class="flex justify-center py-12">
					<span class="loading loading-md loading-spinner"></span>
				</div>
			{:else if detailItem && detailItem.id === selectedItemId}
				<div class="mb-3 flex items-start justify-between">
					<h2 class="text-lg font-bold break-all">{detailItem.filename}</h2>
					<button class="btn btn-ghost btn-xs" onclick={handleCloseDetail}> &times; </button>
				</div>

				<p class="mb-3 text-xs break-all text-base-content/60">{detailItem.path}</p>

				<div class="mb-4 flex flex-wrap gap-2">
					<span class={classNames('badge', cloudAdapter.extensionBadgeClass(detailItem.extension))}>
						{detailItem.extension}
					</span>
					{#if detailItem.sizeBytes}
						<span class="badge badge-neutral">
							{cloudAdapter.formatBytes(detailItem.sizeBytes)}
						</span>
					{/if}
					{#if detailItem.mimeType}
						<span class="badge badge-ghost">{detailItem.mimeType}</span>
					{/if}
				</div>

				<div class="divider"></div>

				<h3 class="mb-2 text-sm font-semibold">Attributes</h3>

				{#if detailItem.attributes.length > 0}
					<div class="space-y-2">
						{#each detailItem.attributes as attr (attr.key + attr.source)}
							<div class="flex items-start justify-between gap-2">
								<div class="min-w-0 flex-1">
									<div class="text-xs font-medium">{attr.key}</div>
									<div class="text-xs break-all text-base-content/70">
										{cloudAdapter.formatAttributeValue(attr.value, attr.typeId)}
									</div>
									<div class="mt-0.5 flex gap-1">
										<span
											class={classNames(
												'badge badge-xs',
												cloudAdapter.attributeTypeBadgeClass(attr.typeId)
											)}
										>
											{attr.typeId}
										</span>
										<span class="badge badge-ghost badge-xs">{attr.source}</span>
									</div>
								</div>
								<button
									class="btn text-error btn-ghost btn-xs"
									onclick={() => handleRemoveAttribute(attr.key)}
								>
									&times;
								</button>
							</div>
						{/each}
					</div>
				{:else}
					<p class="py-2 text-xs text-base-content/60">No attributes yet</p>
				{/if}

				<div class="divider"></div>

				<h4 class="mb-2 text-xs font-semibold">Add Attribute</h4>
				<div class="space-y-2">
					<input
						type="text"
						class="input-bordered input input-xs w-full"
						placeholder="Key"
						bind:value={newAttrKey}
					/>
					<input
						type="text"
						class="input-bordered input input-xs w-full"
						placeholder="Value"
						bind:value={newAttrValue}
					/>
					<div class="flex gap-2">
						<select class="select-bordered select flex-1 select-xs" bind:value={newAttrType}>
							<option value="string">String</option>
							<option value="number">Number</option>
							<option value="boolean">Boolean</option>
							<option value="date">Date</option>
							<option value="url">URL</option>
							<option value="duration">Duration</option>
							<option value="bytes">Bytes</option>
							<option value="tags">Tags</option>
							<option value="json">JSON</option>
						</select>
						<button
							class="btn btn-xs btn-primary"
							disabled={!newAttrKey || !newAttrValue}
							onclick={handleAddAttribute}
						>
							Add
						</button>
					</div>
				</div>

				{#if detailItem.links.length > 0}
					<div class="divider"></div>
					<h3 class="mb-2 text-sm font-semibold">External Links</h3>
					<div class="space-y-2">
						{#each detailItem.links as link (link.service)}
							<div class="flex items-center justify-between">
								<span class="badge badge-sm badge-primary">{link.service}</span>
								<span class="text-xs text-base-content/60">{link.serviceId}</span>
							</div>
						{/each}
					</div>
				{/if}
			{/if}
		</aside>
	{/if}
</div>
