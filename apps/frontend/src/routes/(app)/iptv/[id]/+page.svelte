<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { base } from '$app/paths';
	import IptvChannelDetail from 'ui-lib/components/iptv/IptvChannelDetail.svelte';
	import { iptvService } from 'ui-lib/services/iptv.service';
	import type { IptvChannel, IptvStream, IptvEpgProgram } from 'ui-lib/types/iptv.type';

	let channel = $state<IptvChannel | null>(null);
	let streams = $state<IptvStream[]>([]);
	let streamUrl = $state('');
	let loading = $state(true);
	let epgPrograms = $state<IptvEpgProgram[]>([]);
	let epgAvailable = $state(false);

	let id = $derived($page.params.id ?? '');

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
		onback={() => goto(`${base}/iptv`)}
		onstreamselect={handleStreamSelect}
	/>
{:else if loading}
	<div class="flex flex-1 items-center justify-center">
		<span class="loading loading-lg loading-spinner"></span>
	</div>
{:else}
	<div class="flex flex-1 flex-col items-center justify-center gap-2">
		<p class="text-sm opacity-60">Channel not found</p>
		<button class="btn btn-ghost btn-sm" onclick={() => goto(`${base}/iptv`)}>Back</button>
	</div>
{/if}
