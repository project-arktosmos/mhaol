<script lang="ts">
	import classNames from 'classnames';
	import IptvChannelCard from './IptvChannelCard.svelte';
	import type { IptvChannel, IptvCategory, IptvCountry } from 'ui-lib/types/iptv.type';

	let {
		channels,
		total,
		page,
		categories,
		countries,
		loading = false,
		error = null,
		query = '',
		selectedCategory = '',
		selectedCountry = '',
		epgOnly = false,
		favoritedIds,
		pinnedIds,
		onsearch,
		onfilterchange,
		onpagechange,
		onchannelclick
	}: {
		channels: IptvChannel[];
		total: number;
		page: number;
		categories: IptvCategory[];
		countries: IptvCountry[];
		loading?: boolean;
		error?: string | null;
		query?: string;
		selectedCategory?: string;
		selectedCountry?: string;
		epgOnly?: boolean;
		favoritedIds?: Set<string>;
		pinnedIds?: Set<string>;
		onsearch?: (query: string) => void;
		onfilterchange?: (filters: { category?: string; country?: string; epgOnly?: boolean }) => void;
		onpagechange?: (page: number) => void;
		onchannelclick?: (channel: IptvChannel) => void;
	} = $props();

	let searchInput = $state(query);
	let totalPages = $derived(Math.ceil(total / 50));

	function handleSearch(): void {
		onsearch?.(searchInput);
	}

	function handleKeydown(e: KeyboardEvent): void {
		if (e.key === 'Enter') handleSearch();
	}
</script>

<div class="flex h-full flex-col overflow-hidden">
	<div class="shrink-0 border-b border-base-300 p-4">
		<div class="flex flex-col gap-3 lg:flex-row lg:items-end">
			<div class="flex flex-1 gap-2">
				<input
					type="text"
					class="input-bordered input input-sm flex-1"
					placeholder="Search channels..."
					bind:value={searchInput}
					onkeydown={handleKeydown}
				/>
				<button class="btn btn-sm btn-primary" onclick={handleSearch}>Search</button>
			</div>

			<div class="flex flex-wrap gap-2">
				<select
					class="select-bordered select select-sm"
					value={selectedCategory}
					onchange={(e) => onfilterchange?.({ category: e.currentTarget.value })}
				>
					<option value="">All categories</option>
					{#each categories as cat}
						<option value={cat.id}>{cat.name}</option>
					{/each}
				</select>

				<select
					class="select-bordered select select-sm"
					value={selectedCountry}
					onchange={(e) => onfilterchange?.({ country: e.currentTarget.value })}
				>
					<option value="">All countries</option>
					{#each countries as co}
						<option value={co.code}>{co.name}</option>
					{/each}
				</select>

				<label class="label cursor-pointer gap-2">
					<span class="text-sm">EPG only</span>
					<input
						type="checkbox"
						class="toggle toggle-info toggle-sm"
						checked={epgOnly}
						onchange={(e) => onfilterchange?.({ epgOnly: e.currentTarget.checked })}
					/>
				</label>
			</div>
		</div>

		<div class="mt-2 text-xs opacity-60">
			{total.toLocaleString()} channel{total === 1 ? '' : 's'} found
		</div>
	</div>

	{#if error}
		<div class="m-4">
			<div class="alert alert-error">
				<span>{error}</span>
			</div>
		</div>
	{/if}

	<div class="flex-1 overflow-y-auto p-4">
		{#if loading}
			<div class="flex items-center justify-center py-12">
				<span class="loading loading-lg loading-spinner"></span>
			</div>
		{:else if channels.length === 0}
			<div class="flex items-center justify-center py-12">
				<p class="text-sm opacity-60">No channels found</p>
			</div>
		{:else}
			<div
				class="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6"
			>
				{#each channels as channel (channel.id)}
					<IptvChannelCard
						{channel}
						favorited={favoritedIds?.has(channel.id) ?? false}
						pinned={pinnedIds?.has(channel.id) ?? false}
						onclick={() => onchannelclick?.(channel)}
					/>
				{/each}
			</div>
		{/if}
	</div>

	{#if totalPages > 1}
		<div class="shrink-0 border-t border-base-300 p-3">
			<div class="flex items-center justify-center gap-2">
				<button
					class={classNames('btn btn-sm', { 'btn-disabled': page <= 1 })}
					disabled={page <= 1}
					onclick={() => onpagechange?.(page - 1)}
				>
					Previous
				</button>
				<span class="text-sm">
					Page {page} of {totalPages}
				</span>
				<button
					class={classNames('btn btn-sm', { 'btn-disabled': page >= totalPages })}
					disabled={page >= totalPages}
					onclick={() => onpagechange?.(page + 1)}
				>
					Next
				</button>
			</div>
		</div>
	{/if}
</div>
