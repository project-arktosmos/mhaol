<script lang="ts">
	import classNames from 'classnames';
	import type { Snippet } from 'svelte';
	import DetailPageLayout from '$components/core/DetailPageLayout.svelte';
	import type { CatalogItem } from '$types/catalog.type';
	import { TORRENT_KINDS } from '$types/catalog.type';
	import type { TorrentState } from '$types/torrent.type';
	import {
		formatBytes,
		formatSpeed,
		formatEta,
		getStateLabel,
		getStateColor
	} from '$types/torrent.type';

	interface FetchSteps {
		terms: boolean;
		search: boolean;
		searching: boolean;
		eval: boolean;
		done: boolean;
	}

	interface TorrentStatus {
		state: TorrentState;
		progress: number;
		size: number;
		downloadSpeed: number;
		uploadSpeed: number;
		peers: number;
		seeds: number;
		eta: number | null;
	}

	interface FetchedTorrentEntry {
		label: string;
		name: string;
		quality: string;
		languages: string;
	}

	type SearchLang = 'en' | 'es';

	interface Props {
		item: CatalogItem;
		loading?: boolean;
		fetching?: boolean;
		fetched?: boolean;
		streaming?: boolean;
		fetchSteps?: FetchSteps | null;
		torrentStatus?: TorrentStatus | null;
		fetchedTorrent?: { name: string; quality: string; languages: string } | null;
		fetchedTorrents?: FetchedTorrentEntry[] | null;
		isFavorite?: boolean;
		isPinned?: boolean;
		searchLang?: SearchLang | null;
		onfetch?: () => void;
		ondownload?: () => void;
		onstream?: () => void;
		onshowsearch?: () => void;
		onback: () => void;
		ontogglefavorite?: () => void;
		ontogglepin?: () => void;
		onsearchlangchange?: (lang: SearchLang) => void;
		sidebar?: Snippet;
		extra?: Snippet;
		rightPanel?: Snippet;
	}

	let {
		item,
		loading = false,
		fetching = false,
		fetched = false,
		streaming = false,
		fetchSteps = null,
		torrentStatus = null,
		fetchedTorrent = null,
		fetchedTorrents = null,
		isFavorite = false,
		isPinned = false,
		searchLang = null,
		onfetch,
		ondownload,
		onstream,
		onshowsearch,
		onback,
		ontogglefavorite,
		ontogglepin,
		onsearchlangchange,
		sidebar,
		extra,
		rightPanel
	}: Props = $props();

	let supportsTorrent = $derived(TORRENT_KINDS.includes(item.kind));
	let dlState = $derived(torrentStatus?.state ?? null);
	let isDownloading = $derived(
		dlState === 'downloading' ||
			dlState === 'initializing' ||
			dlState === 'paused' ||
			dlState === 'checking'
	);
	let isDownloaded = $derived(dlState === 'seeding');
	let dlProgress = $derived(torrentStatus?.progress ?? 0);
	let dlPercent = $derived(Math.round(dlProgress * 100));
</script>

<DetailPageLayout>
	<button class="btn self-start btn-ghost btn-sm" onclick={onback}>
		<svg
			xmlns="http://www.w3.org/2000/svg"
			class="h-4 w-4"
			fill="none"
			viewBox="0 0 24 24"
			stroke="currentColor"
			stroke-width="2"
		>
			<path stroke-linecap="round" stroke-linejoin="round" d="M15 19l-7-7 7-7" />
		</svg>
		Back
	</button>

	{#if item.posterUrl}
		<img
			src={item.posterUrl}
			alt="{item.title} poster"
			class="w-full rounded-lg object-cover"
			loading="lazy"
		/>
	{/if}

	{#if sidebar}
		{@render sidebar()}
	{/if}

	{#snippet cellA()}
		{#if loading}
			<div class="flex items-center justify-center py-16">
				<span class="loading loading-lg loading-spinner"></span>
			</div>
		{:else}
			<div class="flex flex-col gap-3">
				<div class="relative">
					{#if rightPanel}
						{@render rightPanel()}
					{:else if item.backdropUrl}
						<img
							src={item.backdropUrl}
							alt="{item.title} backdrop"
							class="w-full rounded-lg object-cover"
							loading="lazy"
						/>
					{/if}

					{#if supportsTorrent && !streaming}
						<div class="absolute inset-0 flex items-center justify-center">
							<div class="flex flex-wrap items-center justify-center gap-2">
								{#if searchLang && onsearchlangchange}
									<select
										class="select-bordered select select-sm shadow-lg"
										value={searchLang}
										onchange={(e) =>
											onsearchlangchange(
												(e.currentTarget as HTMLSelectElement).value as SearchLang
											)}
									>
										<option value="en">English</option>
										<option value="es">Spanish</option>
									</select>
								{/if}
								{#if onfetch}
									<button
										class="btn shadow-lg btn-sm btn-primary"
										disabled={fetching}
										onclick={onfetch}
									>
										{#if fetching}
											<span class="loading loading-xs loading-spinner"></span>
										{/if}
										{fetched ? 'Re-fetch' : 'Fetch'}
									</button>
								{/if}
								{#if ondownload}
									<button
										class="btn shadow-lg btn-sm btn-secondary"
										disabled={!fetched || isDownloading || isDownloaded}
										onclick={ondownload}
									>
										Download
									</button>
								{/if}
								{#if onstream}
									<button class="btn shadow-lg btn-sm btn-accent" onclick={onstream}>
										Stream
									</button>
								{/if}
								{#if onshowsearch}
									<button class="btn shadow-lg btn-ghost btn-sm" onclick={onshowsearch}>
										Manual Search
									</button>
								{/if}
							</div>
						</div>
					{/if}
				</div>

				<h1 class="text-2xl font-bold">{item.title}</h1>

				<div class="flex flex-wrap items-center gap-2">
					{#if item.year}
						<span class="badge badge-outline">{item.year}</span>
					{/if}
					{#if item.voteAverage}
						<span
							class={classNames('badge', {
								'badge-success': item.voteAverage >= 7,
								'badge-warning': item.voteAverage >= 5 && item.voteAverage < 7,
								'badge-error': item.voteAverage < 5
							})}
						>
							{item.voteAverage.toFixed(1)}
						</span>
					{/if}
					{#if ontogglefavorite}
						<button
							class={classNames('btn btn-xs', {
								'btn-error': isFavorite,
								'btn-ghost': !isFavorite
							})}
							onclick={ontogglefavorite}
						>
							{isFavorite ? '♥ Favorited' : '♡ Favorite'}
						</button>
					{/if}
					{#if ontogglepin}
						<button
							class={classNames('btn btn-xs', { 'btn-info': isPinned, 'btn-ghost': !isPinned })}
							onclick={ontogglepin}
						>
							{isPinned ? '📌 Pinned' : '📌 Pin'}
						</button>
					{/if}
				</div>

				{#if item.overview}
					<p class="text-sm leading-relaxed opacity-80">{item.overview}</p>
				{/if}
			</div>
		{/if}
	{/snippet}

	{#snippet cellB()}
		{#if !loading}
			{#if fetchSteps}
				<div class="flex items-center gap-2 text-xs opacity-60">
					<span class={classNames({ 'text-success': fetchSteps.terms })}>Terms</span>
					<span>→</span>
					<span class={classNames({ 'text-success': fetchSteps.search })}>
						{fetchSteps.searching ? 'Searching...' : 'Search'}
					</span>
					<span>→</span>
					<span class={classNames({ 'text-success': fetchSteps.eval })}>Eval</span>
					<span>→</span>
					<span class={classNames({ 'text-success': fetchSteps.done })}>Done</span>
				</div>
			{/if}

			{#if fetchedTorrents && fetchedTorrents.length > 0}
				<div class="flex flex-col gap-2">
					{#each fetchedTorrents as t}
						<div class="rounded-lg bg-base-200 p-3 text-sm">
							<div class="mb-1 flex items-center gap-2">
								<span class="badge badge-xs badge-primary">{t.label}</span>
							</div>
							<p class="font-medium break-words">{t.name}</p>
							<div class="mt-1 flex gap-2 text-xs opacity-60">
								<span>{t.quality}</span>
								<span>{t.languages}</span>
							</div>
						</div>
					{/each}
				</div>
			{:else if fetchedTorrent}
				<div class="rounded-lg bg-base-200 p-3 text-sm">
					<p class="font-medium">{fetchedTorrent.name}</p>
					<div class="mt-1 flex gap-2 text-xs opacity-60">
						<span>{fetchedTorrent.quality}</span>
						<span>{fetchedTorrent.languages}</span>
					</div>
				</div>
			{/if}

			{#if torrentStatus && isDownloading}
				<div class="rounded-lg bg-base-200 p-3">
					<div class="flex items-center justify-between text-sm">
						<span class={getStateColor(torrentStatus.state)}>
							{getStateLabel(torrentStatus.state)}
						</span>
						<span class="font-mono">{dlPercent}%</span>
					</div>
					<progress class="progress mt-1 w-full progress-primary" value={dlProgress} max="1"
					></progress>
					<div class="mt-1 flex gap-3 text-xs opacity-60">
						<span>↓ {formatSpeed(torrentStatus.downloadSpeed)}</span>
						<span>↑ {formatSpeed(torrentStatus.uploadSpeed)}</span>
						<span>{formatBytes(torrentStatus.size)}</span>
						{#if torrentStatus.eta}
							<span>ETA {formatEta(torrentStatus.eta)}</span>
						{/if}
					</div>
				</div>
			{/if}

			{#if isDownloaded}
				<div class="rounded-lg bg-success/10 p-3 text-sm text-success">
					Downloaded — {formatBytes(torrentStatus?.size ?? 0)}
				</div>
			{/if}

			{#if extra}
				{@render extra()}
			{/if}
		{/if}
	{/snippet}
</DetailPageLayout>
