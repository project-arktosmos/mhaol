<script lang="ts">
	import classNames from 'classnames';
	import { iptvAdapter } from 'ui-lib/adapters/classes/iptv.adapter';
	import IptvPlayer from './IptvPlayer.svelte';
	import type { IptvChannel, IptvStream, IptvEpgProgram } from 'ui-lib/types/iptv.type';

	let {
		channel,
		streams,
		streamUrl,
		loading = false,
		epgPrograms = [],
		epgAvailable = false,
		isFavorite = false,
		togglingFavorite = false,
		isPinned = false,
		togglingPin = false,
		onback,
		onstreamselect,
		ontogglefavorite,
		ontogglepin
	}: {
		channel: IptvChannel;
		streams: IptvStream[];
		streamUrl: string;
		loading?: boolean;
		epgPrograms?: IptvEpgProgram[];
		epgAvailable?: boolean;
		isFavorite?: boolean;
		togglingFavorite?: boolean;
		isPinned?: boolean;
		togglingPin?: boolean;
		onback?: () => void;
		onstreamselect?: (stream: IptvStream) => void;
		ontogglefavorite?: () => void;
		ontogglepin?: () => void;
	} = $props();

	let selectedStreamIndex = $state(0);
	let subtitle = $derived(iptvAdapter.formatChannelSubtitle(channel));

	function formatEpgTime(raw: string): string {
		// Format: "20260325125722 +0000"
		if (raw.length < 12) return raw;
		const h = raw.slice(8, 10);
		const m = raw.slice(10, 12);
		return `${h}:${m}`;
	}

	function isNowPlaying(program: IptvEpgProgram): boolean {
		const now = new Date();
		const pad = (n: number) => String(n).padStart(2, '0');
		const nowStr =
			`${now.getUTCFullYear()}${pad(now.getUTCMonth() + 1)}${pad(now.getUTCDate())}` +
			`${pad(now.getUTCHours())}${pad(now.getUTCMinutes())}${pad(now.getUTCSeconds())} +0000`;
		return program.start <= nowStr && program.stop > nowStr;
	}
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
		{#if ontogglefavorite && ontogglepin}
			<div class="flex gap-1">
				<button
					class={classNames('btn btn-sm', { 'btn-error': isFavorite, 'btn-outline': !isFavorite })}
					onclick={ontogglefavorite}
					disabled={togglingFavorite}
				>
					{#if togglingFavorite}
						<span class="loading loading-xs loading-spinner"></span>
					{:else}
						<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill={isFavorite ? 'currentColor' : 'none'} stroke="currentColor" stroke-width="2">
							<path stroke-linecap="round" stroke-linejoin="round" d="M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z" />
						</svg>
					{/if}
				</button>
				<button
					class={classNames('btn btn-sm', { 'btn-info': isPinned, 'btn-outline': !isPinned })}
					onclick={ontogglepin}
					disabled={togglingPin}
				>
					{#if togglingPin}
						<span class="loading loading-xs loading-spinner"></span>
					{:else}
						<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill={isPinned ? 'currentColor' : 'none'} stroke="currentColor" stroke-width="2">
							<path fill-rule="evenodd" d="M10.788 3.21c.448-1.077 1.976-1.077 2.424 0l2.082 5.007 5.404.433c1.164.093 1.636 1.545.749 2.305l-4.117 3.527 1.257 5.273c.271 1.136-.964 2.033-1.96 1.425L12 18.354 7.373 21.18c-.996.608-2.231-.29-1.96-1.425l1.257-5.273-4.117-3.527c-.887-.76-.415-2.212.749-2.305l5.404-.433 2.082-5.006z" clip-rule="evenodd" />
						</svg>
					{/if}
				</button>
			</div>
		{/if}
	</div>

	{#if loading}
		<div class="flex flex-1 items-center justify-center">
			<span class="loading loading-lg loading-spinner"></span>
		</div>
	{:else}
		<div class="flex flex-1 flex-col gap-4 p-4 lg:flex-row">
			<div class="flex-1">
				{#if streamUrl}
					<IptvPlayer src={streamUrl} poster={channel.logo} />
				{:else}
					<div class="flex aspect-video items-center justify-center rounded-lg bg-base-300">
						<p class="text-sm opacity-60">No streams available for this channel</p>
					</div>
				{/if}

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

				{#if epgAvailable && epgPrograms.length > 0}
					<div class="mt-3 rounded-lg bg-base-200 p-4">
						<h3 class="mb-2 text-sm font-bold">Program Guide</h3>
						<div class="flex flex-col gap-1">
							{#each epgPrograms as program, i}
								{@const playing = isNowPlaying(program)}
								<div
									class={classNames('rounded-md p-2 text-sm', {
										'bg-primary/10 ring-1 ring-primary/30': playing,
										'opacity-80': !playing
									})}
								>
									<div class="flex items-center gap-2">
										<span class="shrink-0 font-mono text-xs opacity-50">
											{formatEpgTime(program.start)}
										</span>
										{#if playing}
											<span class="badge badge-xs badge-primary">LIVE</span>
										{/if}
									</div>
									<p class="font-medium">{program.title}</p>
									{#if program.episode}
										<p class="text-xs opacity-50">{program.episode}</p>
									{/if}
									{#if playing && program.description}
										<p class="mt-1 text-xs opacity-60">
											{program.description}
										</p>
									{/if}
								</div>
							{/each}
						</div>
					</div>
				{/if}
			</div>
		</div>
	{/if}
</div>
