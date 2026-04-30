<script lang="ts">
	import { onMount } from 'svelte';
	import classNames from 'classnames';
	import { base } from '$app/paths';
	import DocumentCard from 'ui-lib/components/documents/DocumentCard.svelte';
	import type { CloudDocument } from 'ui-lib/types/document.type';
	import {
		listSources,
		loadGenres,
		loadPopular,
		type CatalogItem,
		type CatalogGenre,
		type CatalogSource
	} from '$lib/catalog.service';
	import type { DocumentSource, DocumentType } from '$lib/documents.service';

	let sources = $state<CatalogSource[]>([]);
	let sourcesError = $state<string | null>(null);

	let addon = $state<string>('');
	let type = $state<string>('');
	let filter = $state<string>('');
	let page = $state<number>(1);

	let genres = $state<CatalogGenre[]>([]);
	let genresLoading = $state(false);
	let genresError = $state<string | null>(null);

	let items = $state<CatalogItem[]>([]);
	let totalPages = $state<number>(1);
	let itemsLoading = $state(false);
	let itemsError = $state<string | null>(null);

	const currentSource = $derived(sources.find((s) => s.id === addon));
	const filterLabel = $derived(currentSource?.filterLabel ?? 'Filter');
	const hasFilter = $derived(currentSource?.hasFilter ?? false);

	interface CatalogTypeButton {
		docType: DocumentType;
		label: string;
		addonId: string;
		catalogType: string;
	}

	const catalogTypeButtons = $derived<CatalogTypeButton[]>(
		sources.flatMap((src) =>
			src.types.map((t) => ({
				docType: mapToDocumentType(src.id, t.id),
				label: t.label,
				addonId: src.id,
				catalogType: t.id
			}))
		)
	);

	function mapToDocumentType(addonId: string, typeId: string): DocumentType {
		if (addonId === 'tmdb' && typeId === 'tv') return 'tv show';
		if (addonId === 'tmdb') return 'movie';
		if (addonId === 'musicbrainz') return 'album';
		if (addonId === 'openlibrary') return 'book';
		if (addonId === 'retroachievements') return 'game';
		return 'movie';
	}

	function mapToDocumentSource(addonId: string): DocumentSource {
		if (addonId === 'tmdb') return 'tmdb';
		if (addonId === 'musicbrainz') return 'musicbrainz';
		if (addonId === 'openlibrary') return 'openlibrary';
		if (addonId === 'retroachievements') return 'retroachievements';
		return 'tmdb';
	}

	function virtualDocument(item: CatalogItem): CloudDocument {
		const docType = mapToDocumentType(addon, type);
		const docSource = mapToDocumentSource(addon);
		const images = [item.posterUrl, item.backdropUrl]
			.filter((url): url is string => Boolean(url))
			.map((url) => ({ url, mimeType: 'image/jpeg', fileSize: 0, width: 0, height: 0 }));
		return {
			id: `virtual:${addon}:${type}:${item.id}`,
			title: item.title,
			artists: [],
			description: item.description ?? '',
			images,
			files: [],
			year: item.year,
			type: docType,
			source: docSource,
			created_at: '',
			updated_at: '',
			version: 0,
			version_hashes: []
		};
	}

	function virtualHref(item: CatalogItem): string {
		const params = new URLSearchParams();
		params.set('addon', addon);
		if (type) params.set('type', type);
		params.set('id', item.id);
		params.set('title', item.title);
		if (item.year !== null && item.year !== undefined) params.set('year', String(item.year));
		if (item.description) params.set('description', item.description);
		if (item.posterUrl) params.set('posterUrl', item.posterUrl);
		if (item.backdropUrl) params.set('backdropUrl', item.backdropUrl);
		return `${base}/catalog/virtual?${params.toString()}`;
	}

	async function refreshGenres() {
		if (!addon || !hasFilter) {
			genres = [];
			filter = '';
			return;
		}
		genresLoading = true;
		genresError = null;
		try {
			genres = await loadGenres(addon, type || undefined);
			if (!genres.some((g) => g.id === filter)) {
				filter = genres[0]?.id ?? '';
			}
		} catch (err) {
			genres = [];
			genresError = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			genresLoading = false;
		}
	}

	async function refreshItems() {
		if (!addon) {
			items = [];
			return;
		}
		itemsLoading = true;
		itemsError = null;
		try {
			const result = await loadPopular(addon, {
				type: type || undefined,
				filter: filter || undefined,
				page
			});
			items = result.items;
			totalPages = result.totalPages;
			page = result.page;
		} catch (err) {
			items = [];
			totalPages = 1;
			itemsError = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			itemsLoading = false;
		}
	}

	async function selectType(button: CatalogTypeButton) {
		if (addon === button.addonId && type === button.catalogType) return;
		addon = button.addonId;
		type = button.catalogType;
		page = 1;
		filter = '';
		await refreshGenres();
		await refreshItems();
	}

	async function onFilterChange() {
		page = 1;
		await refreshItems();
	}

	async function goToPage(next: number) {
		if (next < 1 || next > totalPages || next === page) return;
		page = next;
		await refreshItems();
	}

	onMount(async () => {
		try {
			sources = await listSources();
			if (sources.length > 0) {
				addon = sources[0].id;
				type = sources[0].types[0]?.id ?? '';
				await refreshGenres();
				await refreshItems();
			}
		} catch (err) {
			sourcesError = err instanceof Error ? err.message : 'Unknown error';
		}
	});
</script>

<svelte:head>
	<title>Mhaol Cloud — Catalog</title>
</svelte:head>

<div class="flex min-h-full flex-col gap-6 p-6">
	{#if sourcesError}
		<div class="alert alert-error">
			<span>Could not load catalog sources: {sourcesError}</span>
		</div>
	{/if}

	<section class="card border border-base-content/10 bg-base-200 p-4">
		<div class="overflow-x-auto rounded-box border border-base-content/10">
			<table class="table table-sm">
				<tbody>
					<tr>
						<th class="w-32 align-middle">Type</th>
						<td>
							<div class="flex flex-wrap gap-2">
								{#each catalogTypeButtons as button (button.addonId + ':' + button.catalogType)}
									{@const active = addon === button.addonId && type === button.catalogType}
									<button
										type="button"
										class={classNames('btn btn-sm', {
											'btn-primary': active,
											'btn-outline': !active
										})}
										onclick={() => selectType(button)}
										disabled={catalogTypeButtons.length === 0}
									>
										{button.label}
									</button>
								{/each}
							</div>
						</td>
					</tr>
					{#if hasFilter}
						<tr>
							<th class="w-32 align-middle">{filterLabel}</th>
							<td>
								{#if genresLoading}
									<span class="text-xs text-base-content/60"
										>Loading {filterLabel.toLowerCase()}…</span
									>
								{:else if genresError}
									<span class="text-xs text-error">{genresError}</span>
								{:else if genres.length === 0}
									<span class="text-xs text-base-content/60">No options available</span>
								{:else}
									<select
										class="select-bordered select w-full select-sm"
										bind:value={filter}
										onchange={onFilterChange}
									>
										{#each genres as option (option.id)}
											<option value={option.id}>{option.name}</option>
										{/each}
									</select>
								{/if}
							</td>
						</tr>
					{/if}
				</tbody>
			</table>
		</div>
	</section>

	<section class="flex flex-col gap-3">
		<div class="flex items-center justify-between gap-4">
			<h2 class="text-lg font-semibold">Popular</h2>
			<div class="flex items-center gap-2">
				<button
					class="btn btn-outline btn-xs"
					onclick={() => goToPage(page - 1)}
					disabled={itemsLoading || page <= 1}
				>
					Prev
				</button>
				<span class="text-xs text-base-content/60">Page {page} / {totalPages}</span>
				<button
					class="btn btn-outline btn-xs"
					onclick={() => goToPage(page + 1)}
					disabled={itemsLoading || page >= totalPages}
				>
					Next
				</button>
				<button class="btn btn-outline btn-xs" onclick={refreshItems} disabled={itemsLoading}>
					Refresh
				</button>
			</div>
		</div>

		{#if itemsError}
			<div class="alert alert-error">
				<span>{itemsError}</span>
			</div>
		{/if}

		{#if itemsLoading && items.length === 0}
			<p class="text-sm text-base-content/60">Loading…</p>
		{:else if items.length === 0}
			<p class="text-sm text-base-content/60">No items.</p>
		{:else}
			<div
				class={classNames(
					'grid grid-cols-1 gap-4 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5',
					{ 'opacity-60': itemsLoading }
				)}
			>
				{#each items as item (item.id)}
					<div class="flex flex-col gap-2">
						<DocumentCard document={virtualDocument(item)} />
						<a class="link text-center text-xs link-primary" href={virtualHref(item)}>
							View details →
						</a>
					</div>
				{/each}
			</div>
		{/if}
	</section>
</div>
