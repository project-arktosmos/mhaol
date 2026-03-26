<script lang="ts">
	import type { Snippet } from 'svelte';
	import BrowseHeader from 'ui-lib/components/browse/BrowseHeader.svelte';
	import BrowseGrid from 'ui-lib/components/browse/BrowseGrid.svelte';
	import CatalogCard from './CatalogCard.svelte';
	import { catalogItemToCardData } from 'ui-lib/adapters/classes/catalog-card.adapter';
	import type { CatalogBrowseState, CatalogItem, CatalogCardData } from 'ui-lib/types/catalog.type';

	interface Props {
		browseState: CatalogBrowseState;
		title: string;
		cardOverlays?: (item: CatalogItem) => Partial<CatalogCardData>;
		onsearch: (query: string) => void;
		ontabchange: (tabId: string) => void;
		onpagechange: (page: number) => void;
		onselectitem: (item: CatalogItem) => void;
		filterBar?: Snippet;
	}

	let {
		browseState,
		title,
		cardOverlays,
		onsearch,
		ontabchange,
		onpagechange,
		onselectitem,
		filterBar
	}: Props = $props();

	let searchInput = $state('');
	let searchTimer: ReturnType<typeof setTimeout> | null = null;

	function handleSearchInput(e: Event) {
		const value = (e.target as HTMLInputElement).value;
		searchInput = value;
		if (searchTimer) clearTimeout(searchTimer);
		searchTimer = setTimeout(() => {
			if (value.trim()) {
				onsearch(value.trim());
			} else if (browseState.tabs.length > 0) {
				ontabchange(browseState.tabs[0].id);
			}
		}, 400);
	}

	function handleSearchClear() {
		searchInput = '';
		if (browseState.tabs.length > 0) {
			ontabchange(browseState.tabs[0].id);
		}
	}

	function cardDataFor(item: CatalogItem): CatalogCardData {
		const base = catalogItemToCardData(item);
		if (cardOverlays) {
			return { ...base, ...cardOverlays(item) };
		}
		return base;
	}
</script>

<div class="flex h-full flex-col">
	<BrowseHeader {title} count={browseState.items.length}>
		{#snippet controls()}
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
		{#snippet tabs()}
			{#each browseState.tabs as tab}
				<button
					class="btn btn-xs {browseState.activeTab === tab.id ? 'btn-primary' : 'btn-ghost'}"
					onclick={() => ontabchange(tab.id)}
				>
					{tab.label}
				</button>
			{/each}
		{/snippet}
	</BrowseHeader>

	{#if filterBar}
		<div class="border-b border-base-300 px-4 py-2">
			{@render filterBar()}
		</div>
	{/if}

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
			<CatalogCard card={cardDataFor(catalogItem)} onclick={() => onselectitem(catalogItem)} />
		{/snippet}
	</BrowseGrid>
</div>
