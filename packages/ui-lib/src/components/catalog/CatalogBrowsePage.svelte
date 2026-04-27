<script lang="ts">
	import { hasContext, getContext } from 'svelte';
	import type { Snippet } from 'svelte';
	import BrowseHeader from 'ui-lib/components/browse/BrowseHeader.svelte';
	import BrowseGrid from 'ui-lib/components/browse/BrowseGrid.svelte';
	import Portal from 'ui-lib/components/core/Portal.svelte';
	import PinnedFavoritesSection from './PinnedFavoritesSection.svelte';
	import CatalogCard from './CatalogCard.svelte';
	import { catalogItemToCardData } from 'ui-lib/adapters/classes/catalog-card.adapter';
	import type { CatalogKindStrategy } from 'ui-lib/services/catalog.service';
	import type { CatalogBrowseState, CatalogItem, CatalogCardData } from 'ui-lib/types/catalog.type';
	import { MEDIA_BAR_KEY, type MediaBarContext } from 'ui-lib/types/media-bar.type';

	interface Props {
		browseState: CatalogBrowseState;
		title: string;
		strategy?: CatalogKindStrategy;
		cardOverlays?: (item: CatalogItem) => Partial<CatalogCardData>;
		itemHref?: (item: CatalogItem) => string;
		onsearch: (query: string) => void;
		ontabchange: (tabId: string) => void;
		onpagechange: (page: number) => void;
		onselectitem: (item: CatalogItem) => void;
		filterBar?: Snippet;
		extraControls?: Snippet;
		extraSections?: Snippet;
	}

	let {
		browseState,
		title,
		strategy,
		cardOverlays,
		itemHref,
		onsearch,
		ontabchange,
		onpagechange,
		onselectitem,
		filterBar,
		extraControls,
		extraSections
	}: Props = $props();

	const mediaBar: MediaBarContext | null = hasContext(MEDIA_BAR_KEY)
		? getContext<MediaBarContext>(MEDIA_BAR_KEY)
		: null;

	const LIBRARY_TAB_ID = '__library__';
	let hasLibrary = $derived(!!extraSections);
	let viewMode = $state<'library' | 'browse'>('library');

	$effect(() => {
		if (!hasLibrary && viewMode === 'library') {
			viewMode = 'browse';
		}
	});

	$effect(() => {
		if (mediaBar) {
			const count = viewMode === 'library' ? null : browseState.items.length;
			mediaBar.configure({ title, count });
		}
	});

	let searchInput = $state('');
	let searchTimer: ReturnType<typeof setTimeout> | null = null;

	function handleSearchInput(e: Event) {
		const value = (e.target as HTMLInputElement).value;
		searchInput = value;
		if (searchTimer) clearTimeout(searchTimer);
		searchTimer = setTimeout(() => {
			if (value.trim()) {
				viewMode = 'browse';
				onsearch(value.trim());
			} else if (browseState.tabs.length > 0) {
				if (hasLibrary) {
					viewMode = 'library';
				} else {
					ontabchange(browseState.tabs[0].id);
				}
			}
		}, 400);
	}

	function handleSearchClear() {
		searchInput = '';
		if (hasLibrary) {
			viewMode = 'library';
		} else if (browseState.tabs.length > 0) {
			ontabchange(browseState.tabs[0].id);
		}
	}

	function handleTabClick(tabId: string) {
		if (tabId === LIBRARY_TAB_ID) {
			viewMode = 'library';
			return;
		}
		viewMode = 'browse';
		ontabchange(tabId);
	}

	function cardDataFor(item: CatalogItem): CatalogCardData {
		const base = catalogItemToCardData(item);
		if (cardOverlays) {
			return { ...base, ...cardOverlays(item) };
		}
		return base;
	}

	let activeTabId = $derived(viewMode === 'library' ? LIBRARY_TAB_ID : browseState.activeTab);
</script>

{#snippet searchControls()}
	<div class="relative flex-1">
		<input
			type="text"
			class="input-bordered input input-sm w-full max-w-xs"
			placeholder="Search..."
			value={searchInput}
			oninput={handleSearchInput}
		/>
		{#if searchInput}
			<button
				class="btn absolute top-1 right-1 btn-circle btn-ghost btn-xs"
				onclick={handleSearchClear}
			>
				✕
			</button>
		{/if}
	</div>
{/snippet}

{#snippet tabsList()}
	{#if hasLibrary}
		<button
			class="btn btn-xs {activeTabId === LIBRARY_TAB_ID ? 'btn-primary' : 'btn-ghost'}"
			onclick={() => handleTabClick(LIBRARY_TAB_ID)}
		>
			Library
		</button>
	{/if}
	{#each browseState.tabs as tab}
		<button
			class="btn btn-xs {activeTabId === tab.id ? 'btn-primary' : 'btn-ghost'}"
			onclick={() => handleTabClick(tab.id)}
		>
			{tab.label}
		</button>
	{/each}
{/snippet}

{#if mediaBar}
	<Portal target={mediaBar.controlsTarget}>
		{@render searchControls()}
		{#if extraControls}
			{@render extraControls()}
		{/if}
	</Portal>
	{#if browseState.tabs.length > 0 || hasLibrary}
		<Portal target={mediaBar.tabsTarget}>
			<div class="flex flex-wrap gap-1.5 border-b border-base-300 px-4 py-2">
				{@render tabsList()}
			</div>
		</Portal>
	{/if}
	{#if filterBar && viewMode === 'browse'}
		<Portal target={mediaBar.filterBarTarget}>
			<div class="border-b border-base-300 px-4 py-2">
				{@render filterBar()}
			</div>
		</Portal>
	{/if}
{:else}
	<BrowseHeader {title} count={viewMode === 'library' ? null : browseState.items.length}>
		{#snippet controls()}
			{@render searchControls()}
		{/snippet}
		{#snippet tabs()}
			{@render tabsList()}
		{/snippet}
	</BrowseHeader>

	{#if filterBar && viewMode === 'browse'}
		<div class="border-b border-base-300 px-4 py-2">
			{@render filterBar()}
		</div>
	{/if}
{/if}

{#if viewMode === 'library'}
	{#if strategy?.resolveByIds}
		<PinnedFavoritesSection {strategy} {cardOverlays} {itemHref} {onselectitem} />
	{/if}
	{#if extraSections}
		{@render extraSections()}
	{/if}
{:else}
	<BrowseGrid
		items={browseState.items}
		loading={browseState.loading}
		error={browseState.error}
		page={browseState.page - 1}
		totalPages={browseState.totalPages}
		onpage={(p) => onpagechange(p + 1)}
	>
		{#snippet card(item, _index)}
			{@const catalogItem = item as CatalogItem}
			<CatalogCard
				card={cardDataFor(catalogItem)}
				href={itemHref?.(catalogItem)}
				onclick={() => onselectitem(catalogItem)}
			/>
		{/snippet}
	</BrowseGrid>
{/if}
