<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { base } from '$app/paths';
	import IptvChannelDetail from 'ui-lib/components/iptv/IptvChannelDetail.svelte';
	import { iptvService } from 'ui-lib/services/iptv.service';
	import { favoritesService } from 'ui-lib/services/favorites.service';
	import { pinsService } from 'ui-lib/services/pins.service';
	import type { IptvChannel, IptvStream, IptvEpgProgram } from 'ui-lib/types/iptv.type';

	let channel = $state<IptvChannel | null>(null);
	let streams = $state<IptvStream[]>([]);
	let streamUrl = $state('');
	let loading = $state(true);
	let epgPrograms = $state<IptvEpgProgram[]>([]);
	let epgAvailable = $state(false);
	let togglingFavorite = $state(false);
	let togglingPin = $state(false);

	let id = $derived($page.params.id ?? '');

	const favState = favoritesService.state;
	const pinState = pinsService.state;

	let isFavorite = $derived(
		$favState.items.some((f) => f.service === 'iptv' && f.serviceId === id)
	);
	let isPinned = $derived(
		$pinState.items.some((p) => p.service === 'iptv' && p.serviceId === id)
	);

	async function handleToggleFavorite() {
		if (!channel) return;
		togglingFavorite = true;
		try {
			await favoritesService.toggle('iptv', channel.id, channel.name);
		} finally {
			togglingFavorite = false;
		}
	}

	async function handleTogglePin() {
		if (!channel) return;
		togglingPin = true;
		try {
			await pinsService.toggle('iptv', channel.id, channel.name);
		} finally {
			togglingPin = false;
		}
	}

	async function loadChannel(channelId: string): Promise<void> {
		loading = true;
		const detail = await iptvService.getChannel(channelId);
		if (detail) {
			channel = detail.channel;
			streams = detail.streams;
			if (detail.streams.length > 0) {
				streamUrl = iptvService.getStreamUrl(channelId);
			}
		}
		loading = false;

		// Fetch EPG in background
		const epg = await iptvService.getEpg(channelId);
		if (epg) {
			epgAvailable = epg.available;
			epgPrograms = epg.programs;
		}
	}

	function handleStreamSelect(stream: IptvStream): void {
		streamUrl = iptvService.getStreamUrl(channel?.id ?? id);
	}

	onMount(() => {
		loadChannel(id);
	});
</script>

{#if channel}
	<IptvChannelDetail
		{channel}
		{streams}
		{streamUrl}
		{loading}
		{epgPrograms}
		{epgAvailable}
		{isFavorite}
		{togglingFavorite}
		{isPinned}
		{togglingPin}
		onback={() => goto(`${base}/media/iptv`)}
		onstreamselect={handleStreamSelect}
		ontogglefavorite={handleToggleFavorite}
		ontogglepin={handleTogglePin}
	/>
{:else if loading}
	<div class="flex flex-1 items-center justify-center">
		<span class="loading loading-lg loading-spinner"></span>
	</div>
{:else}
	<div class="flex flex-1 flex-col items-center justify-center gap-2">
		<p class="text-sm opacity-60">Channel not found</p>
		<button class="btn btn-ghost btn-sm" onclick={() => goto(`${base}/media/iptv`)}>Back</button>
	</div>
{/if}
