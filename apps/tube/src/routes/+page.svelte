<script lang="ts">
	import { onMount } from 'svelte';
	import classNames from 'classnames';
	import { apiUrl } from '$lib/api-base';
	import { libraryService } from '$services/library.service';
	import { rightPanelService } from '$services/right-panel.service';
	import { youtubeService } from '$services/youtube.service';
	import { youtubeSearchService } from '$services/youtube-search.service';
	import type {
		YouTubeDownloadProgress,
		YouTubeChannelMeta,
		YouTubeRssVideo,
		YouTubeRssFeedResponse
	} from '$types/youtube.type';
	import { youTubeCardAdapter } from '$adapters/classes/youtube-card.adapter';
	import type { LibraryCardItem } from '$types/library.type';
	import type { YouTubeSearchChannelItem } from '$types/youtube-search.type';
	import LibraryContentGrid from '$components/libraries/LibraryContentGrid.svelte';
	import YouTubeSearchInput from '$components/youtube-search/YouTubeSearchInput.svelte';
	import YouTubeChannelCard from '$components/youtube-search/YouTubeChannelCard.svelte';

	const libState = libraryService.state;
	const ytState = youtubeService.state;
	const searchState = youtubeSearchService.state;

	// Channel types (mirrors channels page)
	interface YouTubeChannel {
		id: string;
		handle: string;
		name: string;
		url: string;
		subscriber_text: string | null;
		image_url: string | null;
		created_at: string;
		updated_at: string;
	}

	interface ChannelTableResponse {
		table: string;
		columns: { name: string; type: string }[];
		rows: YouTubeChannel[];
		pagination: {
			page: number;
			limit: number;
			total: number;
			totalPages: number;
		};
	}

	// Channel state
	let dbChannels: YouTubeChannel[] = $state([]);
	let channelsLoading = $state(true);
	let channelMeta: Record<string, YouTubeChannelMeta> = $state({});

	const CHANNEL_ROWS_INCREMENT = 3;
	let channelColumns = $state(4);
	let channelVisibleRows = $state(1);
	let channelGridEl: HTMLDivElement | undefined = $state();

	let channelMaxSlots = $derived(channelColumns * channelVisibleRows);
	let channelHasMore = $derived(dbChannels.length > channelMaxSlots);
	let channelIsExpanded = $derived(channelVisibleRows > 1);
	let visibleChannels = $derived(
		channelHasMore ? dbChannels.slice(0, channelMaxSlots - 1) : dbChannels.slice(0, channelMaxSlots)
	);
	let channelRemaining = $derived(dbChannels.length - visibleChannels.length);

	async function fetchChannels() {
		channelsLoading = true;
		try {
			const res = await fetch(apiUrl('/api/database/tables/youtube_channels?page=1&limit=50'));
			if (!res.ok) return;
			const data: ChannelTableResponse = await res.json();
			dbChannels = data.rows;
			fetchAllChannelMeta(data.rows);
		} catch {
			// silent on home page
		} finally {
			channelsLoading = false;
		}
	}

	async function fetchAllChannelMeta(rows: YouTubeChannel[]) {
		const initial: Record<string, YouTubeChannelMeta> = {};
		const needsFetch: YouTubeChannel[] = [];
		for (const channel of rows) {
			if (channel.image_url) {
				initial[channel.handle] = {
					channelId: channel.id,
					avatar: channel.image_url!,
					description: '',
					subscriberText: channel.subscriber_text ?? ''
				};
			} else {
				needsFetch.push(channel);
			}
		}
		channelMeta = { ...channelMeta, ...initial };

		for (const channel of needsFetch) {
			if (channelMeta[channel.handle]) continue;
			try {
				const res = await fetch(apiUrl(`/api/youtube/channel-meta?handle=${channel.handle}`));
				if (!res.ok) continue;
				const data: YouTubeChannelMeta = await res.json();
				channelMeta = { ...channelMeta, [channel.handle]: data };
			} catch {
				// silent
			}
		}
	}

	// Expanded channel feeds
	interface ChannelFeed {
		channel: YouTubeChannel;
		videos: LibraryCardItem[];
		loading: boolean;
	}

	let expandedFeeds: ChannelFeed[] = $state([]);

	async function toggleChannelFeed(channel: YouTubeChannel) {
		const idx = expandedFeeds.findIndex((f) => f.channel.id === channel.id);
		if (idx >= 0) {
			expandedFeeds = expandedFeeds.filter((_, i) => i !== idx);
			return;
		}

		const feed: ChannelFeed = { channel, videos: [], loading: true };
		expandedFeeds = [...expandedFeeds, feed];

		try {
			const res = await fetch(apiUrl(`/api/youtube/channel-rss?handle=${channel.handle}`));
			if (!res.ok) throw new Error(`HTTP ${res.status}`);
			const data: YouTubeRssFeedResponse = await res.json();
			expandedFeeds = expandedFeeds.map((f) =>
				f.channel.id === channel.id
					? {
							...f,
							videos: data.videos.map((v) => youTubeCardAdapter.fromRssVideo(v)),
							loading: false
						}
					: f
			);
		} catch {
			expandedFeeds = expandedFeeds.map((f) =>
				f.channel.id === channel.id ? { ...f, loading: false } : f
			);
		}
	}

	function handleFeedVideoClick(video: LibraryCardItem) {
		rightPanelService.open({
			videoId: video.videoId,
			title: video.title,
			thumbnail: video.thumbnailUrl ?? ''
		});
	}

	$effect(() => {
		if (!channelGridEl) return;
		function updateColumns() {
			if (!channelGridEl) return;
			const style = getComputedStyle(channelGridEl);
			const cols = style.getPropertyValue('grid-template-columns').split(' ').length;
			channelColumns = cols;
		}
		const observer = new ResizeObserver(updateColumns);
		observer.observe(channelGridEl);
		updateColumns();
		return () => observer.disconnect();
	});

	const ACTIVE_STATES = ['pending', 'fetching', 'downloading', 'muxing'];

	let cardItems = $derived(
		$libState.content.map(youTubeCardAdapter.fromContent.bind(youTubeCardAdapter))
	);

	let favoriteItems = $derived(
		$libState.favorites.map(youTubeCardAdapter.fromContent.bind(youTubeCardAdapter))
	);

	let videoItems = $derived(cardItems.filter((item) => item.hasVideo));
	let musicItems = $derived(cardItems.filter((item) => item.hasAudio));

	let activeDownloadMap = $derived(
		new Map<string, YouTubeDownloadProgress>(
			$ytState.downloads.filter((d) => ACTIVE_STATES.includes(d.state)).map((d) => [d.videoId, d])
		)
	);

	let isSearchMode = $derived(
		$searchState.searching || $searchState.results.length > 0 || !!$searchState.query
	);

	let localSearchResults = $derived(
		isSearchMode && $searchState.query
			? cardItems.filter((item) => {
					const q = $searchState.query.toLowerCase();
					return (
						item.title.toLowerCase().includes(q) ||
						(item.channelName?.toLowerCase().includes(q) ?? false)
					);
				})
			: []
	);

	let youtubeSearchCardItems = $derived(
		$searchState.results.map(youTubeCardAdapter.fromSearchItem.bind(youTubeCardAdapter))
	);

	let searchItemMap = $derived(new Map($searchState.results.map((item) => [item.videoId, item])));

	onMount(() => {
		libraryService.initialize();
		fetchChannels();
	});

	function handleItemClick(item: LibraryCardItem) {
		rightPanelService.open({
			videoId: item.videoId,
			title: item.title,
			thumbnail: item.thumbnailUrl ?? '',
			uploaderName: item.channelName ?? undefined,
			hasVideo: item.hasVideo,
			hasAudio: item.hasAudio
		});
	}

	function handleSearch(query: string) {
		youtubeSearchService.search(query);
	}

	function handleSearchItemClick(cardItem: LibraryCardItem) {
		const item = searchItemMap.get(cardItem.videoId);
		if (item) {
			rightPanelService.open({
				videoId: item.videoId,
				title: item.title,
				thumbnail: item.thumbnail,
				views: item.views,
				viewsText: item.viewsText,
				publishedText: item.uploadedDate || undefined,
				uploaderName: item.uploaderName || undefined,
				uploaderAvatar: item.uploaderAvatar || undefined,
				uploaderUrl: item.uploaderUrl || undefined,
				uploaderVerified: item.uploaderVerified
			});
		}
	}

	function handleLoadMore() {
		youtubeSearchService.loadMore();
	}

	function handleClear() {
		youtubeSearchService.clearResults();
	}

	let subscribedIds = $derived(new Set(dbChannels.map((c) => c.id)));

	function extractHandle(url: string): string | null {
		if (url.startsWith('/@')) return url.slice(2);
		return null;
	}

	async function handleSubscribe(channel: YouTubeSearchChannelItem) {
		const handle = extractHandle(channel.url);
		if (!handle || !channel.channelId) return;
		await fetch(apiUrl('/api/youtube/channel-subscribe'), {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({
				id: channel.channelId,
				handle,
				name: channel.name,
				url: `https://www.youtube.com${channel.url}`,
				subscriber_text: channel.subscriberText || null,
				image_url: channel.thumbnail || null
			})
		});
		await fetchChannels();
	}
</script>

<div class="container mx-auto p-4">
	<div class="mb-6">
		<YouTubeSearchInput
			query={$searchState.query}
			searching={$searchState.searching}
			onsearch={handleSearch}
		/>
	</div>

	{#if $searchState.error}
		<div class="mb-4 alert alert-error">
			<span>{$searchState.error}</span>
			<button
				class="btn btn-ghost btn-sm"
				onclick={() => youtubeSearchService.state.update((s) => ({ ...s, error: null }))}
			>
				Dismiss
			</button>
		</div>
	{/if}

	{#if isSearchMode}
		{#if localSearchResults.length > 0}
			<LibraryContentGrid
				title="In your library"
				items={localSearchResults}
				{activeDownloadMap}
				onitemclick={handleItemClick}
				classes="mb-6"
			/>
		{/if}

		{#if $searchState.searching}
			<div class="mb-4">
				<h2 class="sticky top-0 z-10 mb-4 rounded-lg bg-base-100 px-3 py-2 text-xl font-semibold">
					YouTube videos
				</h2>
				<div class="flex justify-center py-8">
					<span class="loading loading-lg loading-spinner"></span>
				</div>
			</div>
		{:else if youtubeSearchCardItems.length > 0}
			<LibraryContentGrid
				title="YouTube videos"
				items={youtubeSearchCardItems}
				{activeDownloadMap}
				onitemclick={handleSearchItemClick}
				onloadmore={$searchState.continuation ? handleLoadMore : undefined}
				loadingMore={$searchState.loadingMore}
			/>
		{:else if $searchState.query}
			<div class="mb-4">
				<h2 class="sticky top-0 z-10 mb-4 rounded-lg bg-base-100 px-3 py-2 text-xl font-semibold">
					YouTube videos
				</h2>
				<p class="rounded-lg bg-base-200 p-4 text-center opacity-50">No videos found</p>
			</div>
		{/if}

		<div class="mb-4">
			<h2 class="sticky top-0 z-10 mb-4 rounded-lg bg-base-100 px-3 py-2 text-xl font-semibold">
				YouTube channels
			</h2>
			{#if $searchState.searching}
				<div class="flex justify-center py-8">
					<span class="loading loading-lg loading-spinner"></span>
				</div>
			{:else if $searchState.channels.length > 0}
				<div class="grid grid-cols-2 gap-2 sm:grid-cols-3 lg:grid-cols-4">
					{#each $searchState.channels as channel (channel.channelId)}
						<YouTubeChannelCard
							{channel}
							subscribed={subscribedIds.has(channel.channelId)}
							onsubscribe={handleSubscribe}
						/>
					{/each}
				</div>
			{:else}
				<p class="rounded-lg bg-base-200 p-4 text-center opacity-50">No channels found</p>
			{/if}
		</div>
	{:else if $libState.contentLoading && $libState.content.length === 0}
		<div class="flex justify-center py-12">
			<span class="loading loading-md loading-spinner"></span>
		</div>
	{:else if $libState.contentError}
		<div class="rounded-lg bg-error/10 px-4 py-3 text-error">
			{$libState.contentError}
		</div>
	{:else if $libState.content.length === 0 && dbChannels.length === 0}
		<div class="rounded-lg bg-base-200 p-8 text-center">
			<p class="opacity-50">No downloaded content yet. Search for videos above to get started.</p>
		</div>
	{:else}
		{#if dbChannels.length > 0}
			<div class="mb-4">
				<h2 class="sticky top-0 z-10 mb-4 rounded-lg bg-base-100 px-3 py-2 text-xl font-semibold">
					Channels
				</h2>
				<div bind:this={channelGridEl} class="grid grid-cols-2 gap-2 sm:grid-cols-3 lg:grid-cols-4">
					{#each visibleChannels as channel (channel.id)}
						{@const meta = channelMeta[channel.handle]}
						{@const avatar = channel.image_url || meta?.avatar}
						{@const subscriberText = channel.subscriber_text || meta?.subscriberText}
						{@const isOpen = expandedFeeds.some((f) => f.channel.id === channel.id)}
						<button
							onclick={() => toggleChannelFeed(channel)}
							class={classNames(
								'flex items-center gap-3 rounded-lg p-3 text-left transition-colors',
								{
									'bg-primary/15 hover:bg-primary/20': isOpen,
									'bg-base-200 hover:bg-base-300': !isOpen
								}
							)}
						>
							{#if avatar}
								<img
									src={apiUrl(`/api/youtube/image-proxy?url=${encodeURIComponent(avatar)}`)}
									alt={channel.name}
									class="h-10 w-10 shrink-0 rounded-full object-cover"
									loading="lazy"
								/>
							{:else}
								<div
									class="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-error/10 text-error"
								>
									<svg
										xmlns="http://www.w3.org/2000/svg"
										class="h-5 w-5"
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
								<p class="truncate font-medium">{channel.name}</p>
								<p class="truncate text-sm opacity-50">
									@{channel.handle}{subscriberText ? ` · ${subscriberText}` : ''}
								</p>
							</div>
						</button>
					{/each}
					{#if channelHasMore}
						<button
							class="flex min-h-16 items-center justify-center rounded-lg border border-base-300 text-sm opacity-70 transition-opacity hover:opacity-100"
							onclick={() => (channelVisibleRows += CHANNEL_ROWS_INCREMENT)}
						>
							+{channelRemaining} more
						</button>
					{:else if channelIsExpanded}
						<button
							class="flex min-h-16 items-center justify-center rounded-lg border border-base-300 text-sm opacity-70 transition-opacity hover:opacity-100"
							onclick={() => (channelVisibleRows = 1)}
						>
							Show less
						</button>
					{/if}
				</div>
			</div>
		{/if}

		{#each expandedFeeds as feed (feed.channel.id)}
			{#if feed.loading}
				<div class="mb-4">
					<h2 class="sticky top-0 z-10 mb-4 rounded-lg bg-base-100 px-3 py-2 text-xl font-semibold">
						{feed.channel.name}
					</h2>
					<div class="flex justify-center py-6">
						<span class="loading loading-md loading-spinner"></span>
					</div>
				</div>
			{:else if feed.videos.length > 0}
				<LibraryContentGrid
					title={feed.channel.name}
					items={feed.videos}
					{activeDownloadMap}
					onitemclick={handleFeedVideoClick}
				/>
			{:else}
				<div class="mb-4">
					<h2 class="sticky top-0 z-10 mb-4 rounded-lg bg-base-100 px-3 py-2 text-xl font-semibold">
						{feed.channel.name}
					</h2>
					<p class="rounded-lg bg-base-200 p-4 text-center opacity-50">No videos found</p>
				</div>
			{/if}
		{/each}

		{#if favoriteItems.length > 0}
			<LibraryContentGrid
				title="Favorites"
				items={favoriteItems}
				{activeDownloadMap}
				onitemclick={handleItemClick}
			/>
		{/if}

		{#if videoItems.length > 0}
			<LibraryContentGrid
				title="Videos"
				items={videoItems}
				{activeDownloadMap}
				onitemclick={handleItemClick}
			/>
		{/if}

		{#if musicItems.length > 0}
			<LibraryContentGrid
				title="Music"
				items={musicItems}
				{activeDownloadMap}
				onitemclick={handleItemClick}
			/>
		{/if}

		<LibraryContentGrid
			title="All cached"
			items={cardItems}
			{activeDownloadMap}
			onitemclick={handleItemClick}
		/>
	{/if}
</div>
