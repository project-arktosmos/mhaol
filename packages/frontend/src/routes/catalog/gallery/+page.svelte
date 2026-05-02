<script lang="ts">
	import { onMount } from 'svelte';
	import { base } from '$app/paths';
	import { page as pageStore } from '$app/state';
	import FirkinLibraryGrid from '$components/catalog/FirkinLibraryGrid.svelte';
	import FirkinMetadataLookupModal, {
		type CatalogLookupItem
	} from '$components/firkins/FirkinMetadataLookupModal.svelte';
	import { listSources, type CatalogSource } from '$lib/catalog.service';
	import { firkinsService, metadataSearchAddon, type Firkin } from '$lib/firkins.service';

	const firkinsStore = firkinsService.state;
	const firkinsIncludeAll = firkinsService.includeAll;

	// Same local-* mapping used by the catalog page so the gallery view
	// surfaces both remote/bookmarked items and locally-scanned files.
	const LOCAL_ADDON_FOR: Record<string, string> = {
		'tmdb-movie': 'local-movie',
		'tmdb-tv': 'local-tv',
		musicbrainz: 'local-album'
	};

	let sources = $state<CatalogSource[]>([]);
	let sourcesError = $state<string | null>(null);

	const addon = $derived(pageStore.url.searchParams.get('addon') ?? '');
	const currentSource = $derived(sources.find((s) => s.id === addon));
	const sourceLabel = $derived(currentSource?.label ?? addon);

	const galleryFirkins = $derived<Firkin[]>(
		addon
			? $firkinsStore.firkins
					.filter((d) => d.addon === addon || d.addon === LOCAL_ADDON_FOR[addon])
					.slice()
					.sort((a, b) => b.created_at.localeCompare(a.created_at))
			: []
	);
	const galleryFirkinIds = $derived(galleryFirkins.map((d) => d.id));

	function firkinNeedsMetadata(firkin: Firkin): boolean {
		return firkin.description.trim() === '' || firkin.images.length === 0;
	}

	let metadataTarget = $state<{ firkin: Firkin; addon: string } | null>(null);

	function openMetadataLookup(firkin: Firkin) {
		const addonId = metadataSearchAddon(firkin.addon);
		if (!addonId) return;
		metadataTarget = { firkin, addon: addonId };
	}

	async function applyMetadata(item: CatalogLookupItem) {
		if (!metadataTarget) return;
		const id = metadataTarget.firkin.id;
		await firkinsService.enrich(id, {
			title: item.title,
			year: item.year,
			description: item.description ?? '',
			posterUrl: item.posterUrl,
			backdropUrl: item.backdropUrl
		});
		metadataTarget = null;
		await firkinsService.refresh();
	}

	onMount(() => {
		const stopFirkins = firkinsService.start();
		void (async () => {
			try {
				sources = await listSources();
			} catch (err) {
				sourcesError = err instanceof Error ? err.message : 'Unknown error';
			}
		})();
		return () => {
			stopFirkins();
		};
	});
</script>

<svelte:head>
	<title>Mhaol Cloud — {sourceLabel} gallery</title>
</svelte:head>

<div class="flex min-h-full flex-col gap-4">
	<header class="flex flex-wrap items-center justify-between gap-3">
		<div class="flex items-center gap-3">
			<a
				href={addon ? `${base}/catalog?addon=${encodeURIComponent(addon)}` : `${base}/catalog`}
				class="btn btn-ghost btn-sm"
			>
				← Back to catalog
			</a>
			<h1 class="text-xl font-semibold">
				{sourceLabel ? `${sourceLabel} library` : 'Library gallery'}
			</h1>
			<span class="badge badge-ghost">{galleryFirkins.length}</span>
		</div>
		<label class="flex items-center gap-2 text-xs text-base-content/70">
			<input
				type="checkbox"
				class="toggle toggle-primary toggle-sm"
				checked={$firkinsIncludeAll}
				onchange={(e) =>
					firkinsService.setIncludeAll((e.currentTarget as HTMLInputElement).checked)}
			/>
			<span
				title="Off: only bookmarked firkins. On: every firkin in the local DB, including non-bookmarked browse-cache rows from the /catalog/visit resolver."
			>
				Show all locally-available
			</span>
		</label>
	</header>

	{#if sourcesError}
		<div class="alert alert-warning">
			<span>Could not load catalog sources: {sourcesError}</span>
		</div>
	{/if}

	{#if !addon}
		<p class="text-sm text-base-content/60">
			No addon selected — open the gallery from the <a class="link" href={`${base}/catalog`}
				>catalog page</a
			>.
		</p>
	{:else}
		<FirkinLibraryGrid
			firkinIds={galleryFirkinIds}
			collapsed={false}
			emptyMessage={$firkinsIncludeAll
				? `No firkins for ${sourceLabel} yet.`
				: `No bookmarked ${sourceLabel} items yet — toggle "Show all locally-available" to include browse-cache items.`}
		>
			{#snippet actions(doc)}
				{#if firkinNeedsMetadata(doc) && metadataSearchAddon(doc.addon) !== null}
					<button
						type="button"
						class="btn absolute top-2 right-2 btn-xs btn-primary"
						onclick={() => openMetadataLookup(doc)}
						title="Search the relevant addon and bake matching metadata into this firkin"
					>
						Find metadata
					</button>
				{/if}
			{/snippet}
		</FirkinLibraryGrid>
	{/if}
</div>

{#if metadataTarget}
	<FirkinMetadataLookupModal
		open={metadataTarget !== null}
		addon={metadataTarget.addon}
		initialQuery={metadataTarget.firkin.title}
		firkinTitle={metadataTarget.firkin.title}
		onpick={applyMetadata}
		onclose={() => (metadataTarget = null)}
	/>
{/if}
