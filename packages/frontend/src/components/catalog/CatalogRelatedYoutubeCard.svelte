<script lang="ts">
	import { base } from '$app/paths';
	import { goto } from '$app/navigation';
	import { materializeBrowseFirkin } from '$lib/catalog-firkin';
	import FirkinCard from '$components/firkins/FirkinCard.svelte';
	import type { CloudFirkin } from '$types/firkin.type';

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

	function toFirkin(item: RelatedItem): CloudFirkin {
		const images = item.thumbnail
			? [{ url: item.thumbnail, mimeType: 'image/jpeg', fileSize: 0, width: 0, height: 0 }]
			: [];
		const description = [item.uploaderName, item.durationText, item.viewsText]
			.filter((s) => s && s.length > 0)
			.join(' · ');
		return {
			id: firkinIds[item.videoId] ?? `virtual:youtube-video:${item.videoId}`,
			cid: '',
			title: item.title,
			artists: item.uploaderName ? [{ name: item.uploaderName, role: 'channel' }] : [],
			description,
			images,
			files: [],
			year: null,
			addon: 'youtube-video',
			creator: '',
			created_at: '',
			updated_at: '',
			version: 0,
			version_hashes: [],
			reviews: []
		};
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

<section class="flex flex-col gap-2">
	<h2 class="text-sm font-semibold text-base-content/70 uppercase">Related videos</h2>

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
		<div class="grid grid-cols-2 gap-3">
			{#each visibleItems as item (item.videoId)}
				<a href={hrefFor(item)} onclick={(e) => handleClick(e, item)} class="block no-underline">
					<FirkinCard firkin={toFirkin(item)} />
				</a>
			{/each}
		</div>
	{/if}
</section>
