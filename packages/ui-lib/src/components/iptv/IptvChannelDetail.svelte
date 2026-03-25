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
		onback,
		onstreamselect
	}: {
		channel: IptvChannel;
		streams: IptvStream[];
		streamUrl: string;
		loading?: boolean;
		epgPrograms?: IptvEpgProgram[];
		epgAvailable?: boolean;
		onback?: () => void;
		onstreamselect?: (stream: IptvStream) => void;
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
							<img
								src={channel.logo}
								alt={channel.name}
								class="h-20 w-auto object-contain"
							/>
						</div>
					{/if}

					<h2 class="text-lg font-bold">{channel.name}</h2>

					{#if subtitle}
						<p class="mt-1 text-sm opacity-60">{subtitle}</p>
					{/if}

					{#if channel.categories.length > 0}
						<div class="mt-3 flex flex-wrap gap-1">
							{#each channel.categories as cat}
								<span
									class={classNames(
										'badge badge-sm',
										iptvAdapter.getCategoryBadgeClass(cat)
									)}
								>
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
