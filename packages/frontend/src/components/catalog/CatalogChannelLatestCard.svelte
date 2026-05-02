<script lang="ts">
	import { base } from '$app/paths';

	interface ChannelFeedItem {
		videoId: string;
		title: string;
		link: string;
		thumbnailUrl: string | null;
		publishedAt: string | null;
		description: string | null;
	}

	interface ChannelFeed {
		channelId: string;
		channelTitle: string | null;
		items: ChannelFeedItem[];
	}

	interface Props {
		youtubeUrl: string | null;
		/// Cap the rendered list. The Atom feed itself returns ~15 entries.
		limit?: number;
	}

	let { youtubeUrl, limit = 8 }: Props = $props();

	let feed = $state<ChannelFeed | null>(null);
	let status = $state<'idle' | 'loading' | 'done' | 'error' | 'empty'>('idle');
	let error = $state<string | null>(null);
	let initFor: string | null = null;

	$effect(() => {
		const url = youtubeUrl;
		if (!url) {
			status = 'idle';
			feed = null;
			return;
		}
		if (initFor === url) return;
		initFor = url;
		void load(url);
	});

	async function load(url: string): Promise<void> {
		status = 'loading';
		error = null;
		feed = null;
		try {
			const res = await fetch(
				`${base}/api/ytdl/channel/by-video?url=${encodeURIComponent(url)}`,
				{ cache: 'no-store' }
			);
			if (initFor !== url) return;
			if (!res.ok) {
				let message = `HTTP ${res.status}`;
				try {
					const body = await res.json();
					if (body && typeof body.error === 'string') message = body.error;
				} catch {
					// ignore
				}
				throw new Error(message);
			}
			const body = (await res.json()) as ChannelFeed;
			if (initFor !== url) return;
			feed = body;
			status = body.items.length === 0 ? 'empty' : 'done';
		} catch (err) {
			if (initFor !== url) return;
			error = err instanceof Error ? err.message : 'Unknown error';
			status = 'error';
		}
	}

	function formatRelative(iso: string | null): string {
		if (!iso) return '';
		const t = Date.parse(iso);
		if (!Number.isFinite(t)) return '';
		const delta = Date.now() - t;
		if (delta < 0) return '';
		const minute = 60_000;
		const hour = 60 * minute;
		const day = 24 * hour;
		const week = 7 * day;
		const month = 30 * day;
		const year = 365 * day;
		if (delta < hour) return `${Math.max(1, Math.round(delta / minute))} min ago`;
		if (delta < day) return `${Math.round(delta / hour)} hr ago`;
		if (delta < week) return `${Math.round(delta / day)} day ago`;
		if (delta < month) return `${Math.round(delta / week)} wk ago`;
		if (delta < year) return `${Math.round(delta / month)} mo ago`;
		return `${Math.round(delta / year)} yr ago`;
	}

	const visibleItems = $derived(feed ? feed.items.slice(0, limit) : []);
</script>

<section class="card border border-base-content/10 bg-base-200 p-4">
	<header class="mb-3 flex items-baseline justify-between gap-2">
		<h2 class="text-sm font-semibold text-base-content/70 uppercase">Latest from channel</h2>
		{#if feed?.channelTitle}
			<a
				class="link link-hover truncate text-xs text-base-content/60"
				href={`https://www.youtube.com/channel/${encodeURIComponent(feed.channelId)}`}
				target="_blank"
				rel="noopener noreferrer"
				title={feed.channelTitle}
			>
				{feed.channelTitle}
			</a>
		{/if}
	</header>

	{#if status === 'loading' && !feed}
		<div class="flex items-center gap-2 text-xs text-base-content/60">
			<span class="loading loading-xs loading-spinner"></span>
			<span>Fetching channel feed…</span>
		</div>
	{:else if status === 'error'}
		<p class="text-xs text-error">{error ?? 'Failed to fetch channel feed'}</p>
	{:else if status === 'empty'}
		<p class="text-xs text-base-content/60">Channel feed is empty.</p>
	{:else if visibleItems.length > 0}
		<ul class="flex flex-col gap-3">
			{#each visibleItems as item (item.videoId)}
				<li class="flex gap-2">
					<a
						class="link link-hover flex flex-1 gap-2"
						href={item.link}
						target="_blank"
						rel="noopener noreferrer"
						title={item.title}
					>
						{#if item.thumbnailUrl}
							<img
								src={item.thumbnailUrl}
								alt=""
								loading="lazy"
								class="h-12 w-20 shrink-0 rounded object-cover"
							/>
						{:else}
							<div class="h-12 w-20 shrink-0 rounded bg-base-300"></div>
						{/if}
						<div class="flex min-w-0 flex-1 flex-col">
							<span class="line-clamp-2 text-xs font-medium text-base-content">
								{item.title}
							</span>
							{#if item.publishedAt}
								<span class="mt-auto text-[10px] text-base-content/50">
									{formatRelative(item.publishedAt)}
								</span>
							{/if}
						</div>
					</a>
				</li>
			{/each}
		</ul>
	{/if}
</section>
