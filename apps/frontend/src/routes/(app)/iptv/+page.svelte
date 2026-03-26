<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { base } from '$app/paths';
	import IptvBrowsePage from 'ui-lib/components/iptv/IptvBrowsePage.svelte';
	import { iptvService } from 'ui-lib/services/iptv.service';
	import { favoritesService } from 'ui-lib/services/favorites.service';
	import { pinsService } from 'ui-lib/services/pins.service';
	import type { IptvChannel } from 'ui-lib/types/iptv.type';

	const state = iptvService.state;
	const favState = favoritesService.state;
	const pinState = pinsService.state;

	let favoritedIptvIds = $derived(
		new Set($favState.items.filter((f) => f.service === 'iptv').map((f) => f.serviceId))
	);
	let pinnedIptvIds = $derived(
		new Set($pinState.items.filter((p) => p.service === 'iptv').map((p) => p.serviceId))
	);

	function handleSearch(query: string): void {
		iptvService.setQuery(query);
		iptvService.search();
	}

	function handleFilterChange(filters: { category?: string; country?: string; epgOnly?: boolean }): void {
		if (filters.category !== undefined) iptvService.setCategory(filters.category);
		if (filters.country !== undefined) iptvService.setCountry(filters.country);
		if (filters.epgOnly !== undefined) iptvService.setEpgOnly(filters.epgOnly);
		iptvService.search();
	}

	function handlePageChange(page: number): void {
		iptvService.search(page);
	}

	function handleChannelClick(channel: IptvChannel): void {
		goto(`${base}/iptv/${encodeURIComponent(channel.id)}`);
	}

	onMount(() => {
		iptvService.initialize();
	});
</script>

<IptvBrowsePage
	channels={$state.channels}
	total={$state.total}
	page={$state.page}
	categories={$state.categories}
	countries={$state.countries}
	loading={$state.loading}
	error={$state.error}
	query={$state.query}
	selectedCategory={$state.selectedCategory}
	selectedCountry={$state.selectedCountry}
	epgOnly={$state.epgOnly}
	favoritedIds={favoritedIptvIds}
	pinnedIds={pinnedIptvIds}
	onsearch={handleSearch}
	onfilterchange={handleFilterChange}
	onpagechange={handlePageChange}
	onchannelclick={handleChannelClick}
/>
