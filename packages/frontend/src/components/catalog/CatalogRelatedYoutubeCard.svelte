<script lang="ts">
	import { base } from '$app/paths';
	import { goto } from '$app/navigation';
	import { materializeBrowseFirkin } from '$lib/catalog-firkin';

	interface RelatedItem {
		videoId: string;
		title: string;
		url: string;
		thumbnail: string;
		duration: number;
		durationText: string;
		views: number;
		viewsText: string;
		uploadedDate: string;
		uploaderName: string;
		uploaderUrl: string;
		uploaderVerified: boolean;
	}

	interface RelatedResponse {
		videoId: string;
		items: RelatedItem[];
	}

	interface Props {
		youtubeUrl: string | null;
		/// Cap the rendered list. The InnerTube /next response usually
		/// returns ~20 items (mix of videos + a few playlists/mixes the
		/// backend already filters out).
		limit?: number;
	}

	let { youtubeUrl, limit = 12 }: Props = $props();

	let response = $state<RelatedResponse | null>(null);
	let status = $state<'idle' | 'loading' | 'done' | 'error' | 'empty'>('idle');
	let error = $state<string | null>(null);
	let firkinIds = $state<Record<string, string>>({});
	let initFor: string | null = null;

	$effect(() => {
		const url = youtubeUrl;
		if (!url) {
			status = 'idle';
			response = null;
			firkinIds = {};
			return;
		}
		if (initFor === url) return;
		initFor = url;
		void load(url);
	});

	async function load(url: string): Promise<void> {
		status = 'loading';
		error = null;
		response = null;
		firkinIds = {};
		try {
			const res = await fetch(`${base}/api/ytdl/related?url=${encodeURIComponent(url)}`, {
				cache: 'no-store'
			});
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
			const body = (await res.json()) as RelatedResponse;
			if (initFor !== url) return;
			response = body;
			status = body.items.length === 0 ? 'empty' : 'done';
			void materializeAll(url, body.items.slice(0, limit));
		} catch (err) {
			if (initFor !== url) return;
			error = err instanceof Error ? err.message : 'Unknown error';
			status = 'error';
		}
	}

	async function materializeAll(forUrl: string, list: RelatedItem[]): Promise<void> {
		await Promise.all(
			list.map(async (item) => {
				try {
					const created = await materializeBrowseFirkin({
						addon: 'youtube-video',
						upstreamId: item.videoId,
						title: item.title,
						posterUrl: item.thumbnail
					});
					if (initFor !== forUrl) return;
					firkinIds = { ...firkinIds, [item.videoId]: created.id };
				} catch (err) {
					console.warn('[related-youtube] failed to materialize firkin for', item.videoId, err);
				}
			})
		);
	}

	function hrefFor(item: RelatedItem): string {
		const id = firkinIds[item.videoId];
		return id ? `${base}/catalog/${encodeURIComponent(id)}` : `${base}/catalog/visit`;
	}

	async function handleClick(event: MouseEvent, item: RelatedItem) {
		if (event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) {
			return;
		}
		event.preventDefault();
		let id = firkinIds[item.videoId];
		if (!id) {
			try {
				const created = await materializeBrowseFirkin({
					addon: 'youtube-video',
					upstreamId: item.videoId,
					title: item.title,
					posterUrl: item.thumbnail
				});
				id = created.id;
				firkinIds = { ...firkinIds, [item.videoId]: id };
			} catch (err) {
				console.warn('[related-youtube] click materialize failed for', item.videoId, err);
				return;
			}
		}
		await goto(`${base}/catalog/${encodeURIComponent(id)}`);
	}

	const visibleItems = $derived(response ? response.items.slice(0, limit) : []);
</script>

<section class="card border border-base-content/10 bg-base-200 p-4">
	<header class="mb-3 flex items-baseline justify-between gap-2">
		<h2 class="text-sm font-semibold text-base-content/70 uppercase">Related videos</h2>
	</header>

	{#if status === 'loading' && !response}
		<div class="flex items-center gap-2 text-xs text-base-content/60">
			<span class="loading loading-xs loading-spinner"></span>
			<span>Fetching related videos…</span>
		</div>
	{:else if status === 'error'}
		<p class="text-xs text-error">{error ?? 'Failed to fetch related videos'}</p>
	{:else if status === 'empty'}
		<p class="text-xs text-base-content/60">No related videos returned.</p>
	{:else if visibleItems.length > 0}
		<ul class="flex flex-col gap-3">
			{#each visibleItems as item (item.videoId)}
				<li class="flex gap-2">
					<a
						class="flex flex-1 link gap-2 link-hover"
						href={hrefFor(item)}
						onclick={(e) => handleClick(e, item)}
						title={item.title}
					>
						{#if item.thumbnail}
							<img
								src={item.thumbnail}
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
							<span class="mt-auto flex flex-wrap gap-1 text-[10px] text-base-content/50">
								{#if item.uploaderName}
									<span class="truncate">{item.uploaderName}</span>
								{/if}
								{#if item.durationText}
									<span>· {item.durationText}</span>
								{/if}
								{#if item.viewsText}
									<span>· {item.viewsText}</span>
								{/if}
							</span>
						</div>
					</a>
				</li>
			{/each}
		</ul>
	{/if}
</section>
