<script lang="ts">
	import classNames from 'classnames';
	import { onMount } from 'svelte';
	import { cloudItemService } from 'frontend/services/cloud-item.service';
	import { cloudAdapter } from 'frontend/adapters/classes/cloud.adapter';

	const iState = cloudItemService.state;

	let svc = $derived($iState);
	let results = $derived(svc.searchResults);
	let loading = $derived(svc.searchLoading);
	let distinctKeys = $derived(svc.distinctKeys);

	let searchQuery = $state('');
	let filterKey = $state('');
	let filterValue = $state('');

	onMount(async () => {
		await cloudItemService.fetchDistinctKeys();
	});

	async function handleSearch() {
		if (filterKey && filterValue) {
			await cloudItemService.search(undefined, filterKey, filterValue);
		} else if (searchQuery) {
			await cloudItemService.search(searchQuery);
		}
	}

	function handleKeyChange(key: string) {
		filterKey = key;
		filterValue = '';
		if (key) cloudItemService.fetchDistinctValues(key);
	}
</script>

<div class="p-6">
	<h1 class="mb-6 text-2xl font-bold">Search</h1>

	<div class="card mb-6 bg-base-100 shadow-md">
		<div class="card-body">
			<div class="flex flex-wrap gap-3">
				<input
					type="text"
					class="input-bordered input flex-1"
					placeholder="Search by filename..."
					bind:value={searchQuery}
					onkeydown={(e) => {
						if (e.key === 'Enter') handleSearch();
					}}
				/>
				<button class="btn btn-primary" disabled={loading} onclick={handleSearch}>
					{loading ? 'Searching...' : 'Search'}
				</button>
			</div>

			<div class="divider text-xs">or filter by attribute</div>

			<div class="flex flex-wrap gap-3">
				<select
					class="select-bordered select"
					value={filterKey}
					onchange={(e) => handleKeyChange((e.target as HTMLSelectElement).value)}
				>
					<option value="">Select attribute key...</option>
					{#each distinctKeys as key (key)}
						<option value={key}>{key}</option>
					{/each}
				</select>

				{#if filterKey}
					{@const values = svc.distinctValues[filterKey] ?? []}
					<select class="select-bordered select" bind:value={filterValue}>
						<option value="">Select value...</option>
						{#each values as val (val)}
							<option value={val}>{val}</option>
						{/each}
					</select>
				{/if}
			</div>
		</div>
	</div>

	{#if results.length > 0}
		<div class="overflow-x-auto">
			<table class="table">
				<thead>
					<tr>
						<th>Filename</th>
						<th>Extension</th>
						<th>Size</th>
						<th>Attributes</th>
						<th></th>
					</tr>
				</thead>
				<tbody>
					{#each results as item (item.id)}
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
							<td class="text-sm text-base-content/60">
								{cloudAdapter.formatBytes(item.sizeBytes)}
							</td>
							<td>
								<div class="flex flex-wrap gap-1">
									{#each item.attributes.slice(0, 4) as attr (attr.key + attr.source)}
										<span class="badge badge-ghost badge-xs">{attr.key}: {attr.value}</span>
									{/each}
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
	{:else if !loading}
		<div class="py-12 text-center">
			<p class="text-base-content/60">Search for items by filename or filter by attribute.</p>
		</div>
	{/if}
</div>
