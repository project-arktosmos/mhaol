<script lang="ts">
	import { slide } from 'svelte/transition';
	import classNames from 'classnames';
	import { Icon } from 'cloud-ui';
	import { base } from '$app/paths';
	import { goto } from '$app/navigation';
	import { page as pageStore } from '$app/state';
	import FirkinCard from '$components/firkins/FirkinCard.svelte';
	import type { CloudFirkin } from '$types/firkin.type';
	import {
		listSources,
		loadSearch,
		type CatalogItem,
		type CatalogSource
	} from '$lib/catalog.service';
	import type { FirkinAddon } from '$lib/firkins.service';
	import { onMount } from 'svelte';

	let sources = $state<CatalogSource[]>([]);

	onMount(() => {
		void (async () => {
			try {
				sources = await listSources();
			} catch {
				sources = [];
			}
		})();
	});

	const addon = $derived.by(() => {
		const fromUrl = pageStore.url.searchParams.get('addon') ?? '';
		if (sources.length === 0) return fromUrl;
		if (fromUrl && sources.some((s) => s.id === fromUrl)) return fromUrl;
		return sources[0]?.id ?? '';
	});

	const trimmedQuery = $derived((pageStore.url.searchParams.get('q') ?? '').trim());
	const searchField = $derived<'artist' | 'release'>(
		(pageStore.url.searchParams.get('field') as 'artist' | 'release') ?? 'artist'
	);
	const primaryType = $derived((pageStore.url.searchParams.get('primaryType') ?? '').trim());
	// In "all" mode (catalog root with no addon picked / `addon=all`) the
	// navbar search input acts as a local library filter rather than an
	// upstream catalog search, so the results panel should stay closed.
	const rawAddonParam = $derived(pageStore.url.searchParams.get('addon') ?? '');
	const isAllMode = $derived(rawAddonParam === '' || rawAddonParam === 'all');
	const isOpen = $derived(trimmedQuery.length > 0 && !isAllMode);
	const currentSource = $derived(sources.find((s) => s.id === addon));

	/// MusicBrainz release-group `primary-type` values, in the order they
	/// appear on the toggle row. `''` is the "all kinds" pill (no
	/// `primarytype:` constraint applied to the upstream Lucene query).
	const MB_PRIMARY_TYPES: Array<{ id: string; label: string }> = [
		{ id: '', label: 'All' },
		{ id: 'Album', label: 'Albums' },
		{ id: 'EP', label: 'EPs' },
		{ id: 'Single', label: 'Singles' },
		{ id: 'Broadcast', label: 'Broadcasts' },
		{ id: 'Other', label: 'Other' }
	];

	function selectPrimaryType(next: string) {
		if (next === primaryType) return;
		const url = new URL(pageStore.url);
		if (next) {
			url.searchParams.set('primaryType', next);
		} else {
			url.searchParams.delete('primaryType');
		}
		void goto(`${url.pathname}${url.search}${url.hash}`, {
			keepFocus: true,
			noScroll: true,
			replaceState: true
		});
	}

	let items = $state<CatalogItem[]>([]);
	let pageNum = $state<number>(1);
	let totalPages = $state<number>(1);
	let loading = $state(false);
	let error = $state<string | null>(null);
	let token = 0;

	function virtualFirkin(item: CatalogItem): CloudFirkin {
		const images = [item.posterUrl, item.backdropUrl]
			.filter((url): url is string => Boolean(url))
			.map((url) => ({ url, mimeType: 'image/jpeg', fileSize: 0, width: 0, height: 0 }));
		const artists = item.artistName
			? item.artistName
					.split(/\s*,\s*/)
					.filter((n) => n.length > 0)
					.map((name) => ({ name, role: 'artist' }))
			: [];
		return {
			id: `virtual:${addon}:${item.id}`,
			cid: '',
			title: item.title,
			artists,
			description: item.description ?? '',
			images,
			files: [],
			year: item.year,
			addon: addon as FirkinAddon,
			creator: '',
			created_at: '',
			updated_at: '',
			version: 0,
			version_hashes: [],
			reviews: item.reviews ?? []
		};
	}

	function visitHref(item: CatalogItem): string {
		const params = new URLSearchParams();
		params.set('addon', addon);
		params.set('id', item.id);
		params.set('title', item.title);
		if (item.year !== null && item.year !== undefined) params.set('year', String(item.year));
		if (item.description) params.set('description', item.description);
		if (item.posterUrl) params.set('posterUrl', item.posterUrl);
		if (item.backdropUrl) params.set('backdropUrl', item.backdropUrl);
		if (item.artistName) params.set('artistName', item.artistName);
		if (Array.isArray(item.reviews) && item.reviews.length > 0) {
			params.set('reviews', JSON.stringify(item.reviews));
		}
		return `${base}/catalog/visit?${params.toString()}`;
	}

	async function runSearch(nextPage = 1) {
		if (!addon || !trimmedQuery) {
			items = [];
			totalPages = 1;
			pageNum = 1;
			error = null;
			return;
		}
		const t = ++token;
		loading = true;
		error = null;
		try {
			const result = await loadSearch(addon, trimmedQuery, {
				page: nextPage,
				field: addon === 'musicbrainz' ? searchField : undefined,
				primaryType: addon === 'musicbrainz' && primaryType ? primaryType : undefined
			});
			if (t !== token) return;
			items = result.items;
			totalPages = result.totalPages;
			pageNum = result.page;
		} catch (err) {
			if (t !== token) return;
			items = [];
			totalPages = 1;
			error = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			if (t === token) loading = false;
		}
	}

	async function goToPage(next: number) {
		if (next < 1 || next > totalPages || next === pageNum) return;
		await runSearch(next);
	}

	function closePanel() {
		const url = new URL(pageStore.url);
		url.searchParams.delete('q');
		url.searchParams.delete('field');
		url.searchParams.delete('primaryType');
		void goto(`${base}/${url.search}${url.hash}`, {
			keepFocus: true,
			noScroll: true,
			replaceState: true
		});
	}

	$effect(() => {
		const currentAddon = addon;
		const q = trimmedQuery;
		const f = searchField;
		const pt = primaryType;
		void f;
		void pt;
		if (!currentAddon || !q) {
			token++;
			items = [];
			totalPages = 1;
			pageNum = 1;
			error = null;
			loading = false;
			return;
		}
		void runSearch(1);
	});
</script>

<svelte:window
	onkeydown={(e) => {
		if (e.key === 'Escape' && isOpen) closePanel();
	}}
/>

{#if isOpen}
	<div
		class="absolute inset-0 z-30 flex flex-col overflow-hidden bg-base-100"
		transition:slide={{ duration: 200, axis: 'y' }}
	>
		<div
			class="flex items-center justify-between gap-4 border-b border-base-content/10 bg-base-200 px-6 py-3"
		>
			<div class="flex min-w-0 flex-col gap-0.5">
				<h2 class="truncate text-lg font-semibold">
					Search results for "{trimmedQuery}"
				</h2>
				<p class="text-xs text-base-content/60">
					{currentSource?.label ?? addon}
					{#if !loading && items.length > 0}
						· Page {pageNum} / {totalPages}
					{/if}
				</p>
			</div>
			<div class="flex items-center gap-2">
				<button
					class="btn btn-outline btn-xs"
					onclick={() => goToPage(pageNum - 1)}
					disabled={loading || pageNum <= 1}
				>
					Prev
				</button>
				<button
					class="btn btn-outline btn-xs"
					onclick={() => goToPage(pageNum + 1)}
					disabled={loading || pageNum >= totalPages}
				>
					Next
				</button>
				<button
					type="button"
					class="btn btn-circle btn-ghost btn-sm"
					onclick={closePanel}
					title="Close search (Esc)"
					aria-label="Close search"
				>
					<Icon name="lorc/cross-mark" size={18} />
				</button>
			</div>
		</div>

		<div class="flex-1 overflow-y-auto p-6">
			{#if addon === 'musicbrainz'}
				<div
					class="mb-4 flex flex-wrap items-center gap-2"
					role="tablist"
					aria-label="Filter by release type"
				>
					{#each MB_PRIMARY_TYPES as pill (pill.id)}
						{@const active = primaryType === pill.id}
						<button
							type="button"
							role="tab"
							aria-selected={active}
							class={classNames('btn btn-xs', {
								'btn-primary': active,
								'border border-base-content/20 btn-ghost': !active
							})}
							onclick={() => selectPrimaryType(pill.id)}
							disabled={loading}
						>
							{pill.label}
						</button>
					{/each}
				</div>
			{/if}
			{#if error}
				<div class="alert alert-error">
					<span>{error}</span>
				</div>
			{:else if loading && items.length === 0}
				<p class="text-sm text-base-content/60">Searching…</p>
			{:else if items.length === 0}
				<p class="text-sm text-base-content/60">No matches.</p>
			{:else}
				<div
					class={classNames(
						'grid grid-cols-1 gap-4 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5',
						{ 'opacity-60': loading }
					)}
				>
					{#each items as item (item.id)}
						<a
							href={visitHref(item)}
							class="block no-underline"
							onclick={(e) => {
								if ((e.target as HTMLElement).closest('button, summary')) {
									e.preventDefault();
								}
							}}
						>
							<FirkinCard firkin={virtualFirkin(item)} />
						</a>
					{/each}
				</div>
			{/if}
		</div>
	</div>
{/if}
