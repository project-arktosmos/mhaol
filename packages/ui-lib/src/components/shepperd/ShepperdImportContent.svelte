<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import classNames from 'classnames';
	import { smartPairService } from 'ui-lib/services/smart-pair.service';
	import type { SmartPairResult } from 'ui-lib/types/smart-pair.type';

	interface ShepperdItem {
		title: string;
		id: string;
		category: string;
		mediaType: string | null;
		source: string;
		artist?: string;
		imageUrl?: string;
	}

	const TMDB_IMAGE_BASE = 'https://image.tmdb.org/t/p';

	const typeLabels: Record<string, string> = {
		movies: 'Movies',
		tv: 'TV',
		music: 'Music'
	};

	const confidenceBadge: Record<string, string> = {
		high: 'badge-success',
		medium: 'badge-warning',
		low: 'badge-error',
		none: 'badge-ghost'
	};

	const tmdbTypeBadge: Record<string, string> = {
		movie: 'badge-primary',
		tv: 'badge-info'
	};

	const tmdbTypeLabel: Record<string, string> = {
		movie: 'Movie',
		tv: 'TV'
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

	const pairStore = smartPairService.store;

	// Map sourceId -> pair result for fast lookup per table row
	let pairMap = $derived(new Map($pairStore.results.map((r) => [`${r.source}::${r.sourceId}`, r])));

	let matchedCount = $derived($pairStore.results.filter((r) => r.matched).length);

	function getPairResult(item: ShepperdItem): SmartPairResult | undefined {
		return pairMap.get(`${item.source}::${item.id}`);
	}

	function handleSmartPair() {
		smartPairService.pair(filtered.map((i) => ({ title: i.title, id: i.id, source: i.source })));
	}

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
		<div class="flex gap-2">
			{#if extensionDetected && filtered.length > 0}
				<button
					class="btn btn-sm btn-secondary"
					onclick={handleSmartPair}
					disabled={$pairStore.pairing}
				>
					{#if $pairStore.pairing}
						<span class="loading loading-xs loading-spinner"></span>
					{/if}
					Smart Pair
				</button>
			{/if}
			<button class="btn btn-sm btn-primary" onclick={requestCatalog} disabled={loading}>
				{#if loading}
					<span class="loading loading-xs loading-spinner"></span>
				{/if}
				Refresh
			</button>
		</div>
	</div>

	{#if $pairStore.error}
		<div class="alert alert-error">
			<span>{$pairStore.error}</span>
			<button class="btn btn-ghost btn-sm" onclick={() => smartPairService.reset()}>Dismiss</button>
		</div>
	{/if}

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

			{#if $pairStore.pairing}
				<span class="loading loading-sm loading-spinner text-secondary"></span>
				<span class="text-sm text-base-content/60">
					{$pairStore.results.length} paired ({matchedCount} matched)
				</span>
			{:else if $pairStore.results.length > 0}
				<span class="badge badge-secondary">
					{matchedCount}/{$pairStore.results.length} matched
				</span>
			{/if}

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
						<th>Category</th>
						<th>TMDB Match</th>
					</tr>
				</thead>
				<tbody>
					{#each filtered as item (item.source + '::' + item.id)}
						{@const pair = getPairResult(item)}
						<tr>
							<td class="w-12">
								{#if pair?.tmdbPosterPath}
									<img
										src="{TMDB_IMAGE_BASE}/w92{pair.tmdbPosterPath}"
										alt=""
										class="h-10 w-7 rounded object-cover"
									/>
								{:else if item.imageUrl}
									<img src={item.imageUrl} alt="" class="h-8 w-8 rounded object-cover" />
								{/if}
							</td>
							<td class="font-medium">{item.title}</td>
							<td>
								{#if pair?.tmdbType}
									<span class={classNames('badge badge-sm', tmdbTypeBadge[pair.tmdbType] ?? '')}>
										{tmdbTypeLabel[pair.tmdbType] ?? pair.tmdbType}
									</span>
								{:else if item.mediaType}
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
							<td class="text-base-content/60">{item.category}</td>
							<td>
								{#if pair}
									{#if pair.matched}
										<div class="flex items-center gap-2">
											<span class="text-sm font-medium">{pair.tmdbTitle}</span>
											{#if pair.tmdbYear}
												<span class="text-xs text-base-content/50">({pair.tmdbYear})</span>
											{/if}
											<span
												class={classNames('badge badge-xs', confidenceBadge[pair.confidence] ?? '')}
											>
												{pair.confidence}
											</span>
										</div>
									{:else}
										<span class="text-xs text-base-content/40 italic">no match</span>
									{/if}
								{/if}
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{/if}
</div>
