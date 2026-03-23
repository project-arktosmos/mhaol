<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import classNames from 'classnames';

	interface ShepperdItem {
		title: string;
		id: string;
		category: string;
		mediaType: string | null;
		source: string;
		artist?: string;
		imageUrl?: string;
	}

	const typeLabels: Record<string, string> = {
		movies: 'Movies',
		tv: 'TV',
		music: 'Music'
	};

	let items = $state<ShepperdItem[]>([]);
	let loading = $state(false);
	let extensionDetected = $state(false);
	let filterSource = $state<string | null>(null);
	let filterType = $state<string | null>(null);

	let sources = $derived([...new Set(items.map((i) => i.source))].sort());
	let mediaTypes = $derived(
		[...new Set(items.map((i) => i.mediaType).filter(Boolean))].sort() as string[]
	);
	let filtered = $derived(
		items.filter(
			(i) =>
				(!filterSource || i.source === filterSource) && (!filterType || i.mediaType === filterType)
		)
	);

	let timeoutId: ReturnType<typeof setTimeout> | null = null;

	function handleMessage(event: MessageEvent) {
		if (event.source !== window || event.data?.type !== 'SHEPPERD_CATALOG_RESPONSE') return;
		extensionDetected = true;
		items = event.data.items ?? [];
		loading = false;
		if (timeoutId) {
			clearTimeout(timeoutId);
			timeoutId = null;
		}
	}

	function requestCatalog() {
		loading = true;
		extensionDetected = false;
		window.postMessage({ type: 'SHEPPERD_GET_CATALOG' }, '*');
		timeoutId = setTimeout(() => {
			if (!extensionDetected) {
				loading = false;
			}
		}, 2000);
	}

	onMount(() => {
		window.addEventListener('message', handleMessage);
		requestCatalog();
	});

	onDestroy(() => {
		window.removeEventListener('message', handleMessage);
		if (timeoutId) clearTimeout(timeoutId);
	});
</script>

<div class="flex flex-col gap-4">
	<div class="flex items-center justify-between">
		<h2 class="text-lg font-bold">Shepperd Import</h2>
		<button class="btn btn-sm btn-primary" onclick={requestCatalog} disabled={loading}>
			{#if loading}
				<span class="loading loading-xs loading-spinner"></span>
			{/if}
			Refresh
		</button>
	</div>

	{#if loading}
		<div class="flex items-center justify-center py-8">
			<span class="loading loading-md loading-spinner text-primary"></span>
		</div>
	{:else if !extensionDetected}
		<div class="alert alert-warning">
			<span
				>Shepperd browser extension not detected. Make sure it is installed and enabled in this
				browser.</span
			>
		</div>
	{:else if items.length === 0}
		<div class="alert alert-info">
			<span
				>No items captured yet. Use the Shepperd extension to browse supported sites and capture
				your media library.</span
			>
		</div>
	{:else}
		<div class="flex flex-wrap items-center gap-2">
			<span class="badge badge-primary">{filtered.length} of {items.length} items</span>

			<select
				class="select-bordered select select-sm"
				value={filterSource ?? ''}
				onchange={(e) => (filterSource = e.currentTarget.value || null)}
			>
				<option value="">All sources</option>
				{#each sources as src}
					<option value={src}>{src}</option>
				{/each}
			</select>

			<select
				class="select-bordered select select-sm"
				value={filterType ?? ''}
				onchange={(e) => (filterType = e.currentTarget.value || null)}
			>
				<option value="">All types</option>
				{#each mediaTypes as mt}
					<option value={mt}>{typeLabels[mt] ?? mt}</option>
				{/each}
			</select>
		</div>

		<div class="overflow-x-auto rounded-lg border border-base-300">
			<table class="table table-zebra table-sm">
				<thead>
					<tr>
						<th></th>
						<th>Title</th>
						<th>Type</th>
						<th>Source</th>
						<th>Artist</th>
						<th>Category</th>
					</tr>
				</thead>
				<tbody>
					{#each filtered as item (item.source + '::' + item.id)}
						<tr>
							<td class="w-12">
								{#if item.imageUrl}
									<img src={item.imageUrl} alt="" class="h-8 w-8 rounded object-cover" />
								{/if}
							</td>
							<td class="font-medium">{item.title}</td>
							<td>
								{#if item.mediaType}
									<span
										class={classNames('badge badge-sm', {
											'badge-primary': item.mediaType === 'movies',
											'badge-info': item.mediaType === 'tv',
											'badge-accent': item.mediaType === 'music'
										})}
									>
										{typeLabels[item.mediaType] ?? item.mediaType}
									</span>
								{/if}
							</td>
							<td class="text-base-content/60">{item.source}</td>
							<td class="text-base-content/60">{item.artist ?? ''}</td>
							<td class="text-base-content/60">{item.category}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{/if}
</div>
