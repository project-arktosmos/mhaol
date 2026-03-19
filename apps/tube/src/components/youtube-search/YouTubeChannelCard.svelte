<script lang="ts">
	import classNames from 'classnames';
	import { apiUrl } from '$lib/api-base';
	import type { YouTubeSearchChannelItem } from '$types/youtube-search.type';

	let {
		channel,
		subscribed = false,
		onclick,
		onsubscribe
	}: {
		channel: YouTubeSearchChannelItem;
		subscribed?: boolean;
		onclick?: (channel: YouTubeSearchChannelItem) => void;
		onsubscribe?: (channel: YouTubeSearchChannelItem) => void;
	} = $props();

	let subscribeBtnClasses = $derived(
		classNames('btn btn-xs shrink-0', {
			'btn-success': subscribed,
			'btn-outline btn-primary': !subscribed
		})
	);
</script>

<div
	class="flex w-full items-center gap-3 rounded-lg bg-base-200 p-3 transition-colors hover:bg-base-300"
>
	<button
		class="flex min-w-0 flex-1 items-center gap-3 text-left"
		onclick={() => onclick?.(channel)}
	>
		{#if channel.thumbnail}
			<img
				src={apiUrl(`/api/youtube/image-proxy?url=${encodeURIComponent(channel.thumbnail)}`)}
				alt={channel.name}
				class="h-12 w-12 shrink-0 rounded-full object-cover"
				loading="lazy"
			/>
		{:else}
			<div
				class="flex h-12 w-12 shrink-0 items-center justify-center rounded-full bg-error/10 text-error"
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class="h-6 w-6"
					viewBox="0 0 24 24"
					fill="currentColor"
				>
					<path
						d="M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.017 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.015 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.545 15.568V8.432L15.818 12l-6.273 3.568z"
					/>
				</svg>
			</div>
		{/if}

		<div class="min-w-0 flex-1">
			<div class="flex items-center gap-1">
				<p class="truncate font-medium">{channel.name}</p>
				{#if channel.verified}
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="h-4 w-4 shrink-0 text-primary"
						viewBox="0 0 24 24"
						fill="currentColor"
					>
						<path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41L9 16.17z" />
					</svg>
				{/if}
			</div>
			<p class="truncate text-sm opacity-50">
				{#if channel.subscriberText}{channel.subscriberText}{/if}{#if channel.subscriberText && channel.videoCountText}
					·
				{/if}{#if channel.videoCountText}{channel.videoCountText}{/if}
			</p>
			{#if channel.description}
				<p class="mt-1 line-clamp-2 text-xs opacity-60">{channel.description}</p>
			{/if}
		</div>
	</button>

	{#if onsubscribe}
		<button class={subscribeBtnClasses} onclick={() => onsubscribe?.(channel)}>
			{subscribed ? 'Subscribed' : 'Subscribe'}
		</button>
	{/if}
</div>
