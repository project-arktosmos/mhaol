<script lang="ts">
	import classNames from 'classnames';
	import { iptvAdapter } from 'ui-lib/adapters/classes/iptv.adapter';
	import IptvPlayer from './IptvPlayer.svelte';
	import type { IptvChannel, IptvStream } from 'ui-lib/types/iptv.type';

	let {
		channel,
		streams,
		streamUrl,
		loading = false,
		onback,
		onstreamselect
	}: {
		channel: IptvChannel;
		streams: IptvStream[];
		streamUrl: string;
		loading?: boolean;
		onback?: () => void;
		onstreamselect?: (stream: IptvStream) => void;
	} = $props();

	let selectedStreamIndex = $state(0);
	let subtitle = $derived(iptvAdapter.formatChannelSubtitle(channel));
</script>

<div class="flex h-full flex-col overflow-y-auto">
	<div class="flex items-center gap-2 border-b border-base-300 p-4">
		{#if onback}
			<button class="btn btn-ghost btn-sm" onclick={onback}>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class="h-5 w-5"
					fill="none"
					viewBox="0 0 24 24"
					stroke="currentColor"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M15 19l-7-7 7-7"
					/>
				</svg>
				Back
			</button>
		{/if}
		<h1 class="flex-1 truncate text-lg font-bold">{channel.name}</h1>
	</div>

	{#if loading}
		<div class="flex flex-1 items-center justify-center">
			<span class="loading loading-lg loading-spinner"></span>
		</div>
	{:else}
		<div class="flex flex-1 flex-col gap-4 p-4 lg:flex-row">
			<div class="flex-1">
				<IptvPlayer src={streamUrl} poster={channel.logo} />

				{#if streams.length > 1}
					<div class="mt-3">
						<h3 class="mb-1 text-sm font-medium opacity-70">Available streams</h3>
						<div class="flex flex-col gap-1">
							{#each streams as stream, i}
								<button
									type="button"
									class={classNames('btn justify-start text-left btn-sm', {
										'btn-primary': i === selectedStreamIndex,
										'btn-ghost': i !== selectedStreamIndex
									})}
									onclick={() => {
										selectedStreamIndex = i;
										onstreamselect?.(stream);
									}}
								>
									<span class="truncate">Stream {i + 1}</span>
								</button>
							{/each}
						</div>
					</div>
				{/if}
			</div>

			<div class="w-full shrink-0 lg:w-80">
				<div class="rounded-lg bg-base-200 p-4">
					{#if channel.logo}
						<div class="mb-3 flex justify-center">
							<img src={channel.logo} alt={channel.name} class="h-20 w-auto object-contain" />
						</div>
					{/if}

					<h2 class="text-lg font-bold">{channel.name}</h2>

					{#if subtitle}
						<p class="mt-1 text-sm opacity-60">{subtitle}</p>
					{/if}

					{#if channel.categories.length > 0}
						<div class="mt-3 flex flex-wrap gap-1">
							{#each channel.categories as cat}
								<span class={classNames('badge badge-sm', iptvAdapter.getCategoryBadgeClass(cat))}>
									{cat}
								</span>
							{/each}
						</div>
					{/if}

	
					{#if channel.website}
						<div class="mt-3">
							<a
								href={channel.website}
								target="_blank"
								rel="noopener noreferrer"
								class="link text-sm link-primary"
							>
								Visit website
							</a>
						</div>
					{/if}
				</div>
			</div>
		</div>
	{/if}
</div>
