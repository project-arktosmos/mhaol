<script lang="ts">
	import classNames from 'classnames';
	import { iptvAdapter } from 'ui-lib/adapters/classes/iptv.adapter';
	import type { IptvChannel } from 'ui-lib/types/iptv.type';

	let {
		channel,
		onclick
	}: {
		channel: IptvChannel;
		onclick?: () => void;
	} = $props();

	let subtitle = $derived(iptvAdapter.formatChannelSubtitle(channel));
</script>

<button
	type="button"
	class={classNames(
		'card-compact card cursor-pointer bg-base-200 transition-shadow hover:shadow-md',
		{ 'opacity-60': channel.isNsfw }
	)}
	{onclick}
>
	<figure class="relative flex h-32 items-center justify-center overflow-hidden bg-base-300">
		{#if channel.hasEpg}
			<span class="badge badge-xs badge-info absolute right-1 top-1">EPG</span>
		{/if}
		{#if channel.logo}
			<img
				src={channel.logo}
				alt={channel.name}
				class="h-full w-full object-contain p-4"
				loading="lazy"
			/>
		{:else}
			<svg
				xmlns="http://www.w3.org/2000/svg"
				class="h-12 w-12 opacity-20"
				fill="none"
				viewBox="0 0 24 24"
				stroke="currentColor"
			>
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="2"
					d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"
				/>
			</svg>
		{/if}
	</figure>
	<div class="card-body gap-1">
		<h3 class="card-title text-sm" title={channel.name}>
			<span class="truncate">{channel.name}</span>
		</h3>
		{#if subtitle}
			<p class="truncate text-xs opacity-60">{subtitle}</p>
		{/if}
		{#if channel.categories.length > 0}
			<div class="flex flex-wrap gap-1">
				{#each channel.categories.slice(0, 3) as cat}
					<span class={classNames('badge badge-xs', iptvAdapter.getCategoryBadgeClass(cat))}>
						{cat}
					</span>
				{/each}
			</div>
		{/if}
	</div>
</button>
