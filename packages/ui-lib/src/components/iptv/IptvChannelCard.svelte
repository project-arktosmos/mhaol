<script lang="ts">
	import classNames from 'classnames';
	import { iptvAdapter } from 'ui-lib/adapters/classes/iptv.adapter';
	import type { IptvChannel } from 'ui-lib/types/iptv.type';

	let {
		channel,
		favorited = false,
		pinned = false,
		onclick
	}: {
		channel: IptvChannel;
		favorited?: boolean;
		pinned?: boolean;
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
			<span class="absolute top-1 right-1 badge badge-xs badge-info">EPG</span>
		{/if}
		{#if favorited || pinned}
			<div class="absolute bottom-1.5 left-1.5 z-10 flex gap-1">
				{#if favorited}
					<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-red-500 drop-shadow" viewBox="0 0 24 24" fill="currentColor" stroke="currentColor" stroke-width="2">
						<path stroke-linecap="round" stroke-linejoin="round" d="M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z" />
					</svg>
				{/if}
				{#if pinned}
					<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-blue-400 drop-shadow" viewBox="0 0 24 24" fill="currentColor" stroke="currentColor" stroke-width="2">
						<path fill-rule="evenodd" d="M10.788 3.21c.448-1.077 1.976-1.077 2.424 0l2.082 5.007 5.404.433c1.164.093 1.636 1.545.749 2.305l-4.117 3.527 1.257 5.273c.271 1.136-.964 2.033-1.96 1.425L12 18.354 7.373 21.18c-.996.608-2.231-.29-1.96-1.425l1.257-5.273-4.117-3.527c-.887-.76-.415-2.212.749-2.305l5.404-.433 2.082-5.006z" clip-rule="evenodd" />
					</svg>
				{/if}
			</div>
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
