<script lang="ts">
	import classNames from 'classnames';
	import Modal from 'ui-lib/components/core/Modal.svelte';

	// Mirrors `CatalogItem` in apps/cloud/web/src/lib/catalog.service.ts.
	// Re-declared here so this modal stays in ui-lib without taking a
	// cross-app type dependency.
	export interface CatalogLookupItem {
		id: string;
		title: string;
		year: number | null;
		description: string | null;
		posterUrl: string | null;
		backdropUrl: string | null;
	}

	interface Props {
		open: boolean;
		// Catalog/remote addon to search (e.g. tmdb-movie, musicbrainz). The
		// caller resolves it via `metadataSearchAddon(firkin.addon)`.
		addon: string;
		// Pre-filled into the search box on open. Usually the firkin's
		// current title — the parsed-from-filename one for local-* firkins.
		initialQuery: string;
		// Shown in the header so the user can see which firkin they're
		// enriching while looking at the result list.
		firkinTitle: string;
		// Called when the user picks a result. The parent runs the actual
		// `firkinsService.enrich()` call and handles navigation — keeps this
		// modal portable across pages with different post-enrich behaviour.
		onpick: (item: CatalogLookupItem) => void | Promise<void>;
		onclose: () => void;
	}

	let { open, addon, initialQuery, firkinTitle, onpick, onclose }: Props = $props();

	let query = $state('');
	let searching = $state(false);
	let results = $state<CatalogLookupItem[]>([]);
	let error = $state<string | null>(null);
	let applyingId = $state<string | null>(null);

	// Re-init and auto-search whenever the modal transitions from closed to
	// open (possibly for a different firkin than last time). Only `open` is
	// tracked — `initialQuery` may keep updating while the modal is mounted
	// (the parent's firkin store polls every few seconds), and we don't want
	// to clobber the user's in-progress edits each tick.
	let wasOpen = false;
	$effect(() => {
		if (open && !wasOpen) {
			query = initialQuery;
			results = [];
			error = null;
			applyingId = null;
			void runSearch();
		}
		wasOpen = open;
	});

	async function runSearch() {
		const q = query.trim();
		if (!q) {
			results = [];
			return;
		}
		searching = true;
		error = null;
		try {
			const params = new URLSearchParams({ query: q });
			const res = await fetch(
				`/api/catalog/${encodeURIComponent(addon)}/search?${params.toString()}`,
				{ cache: 'no-store' }
			);
			if (!res.ok) {
				let message = `HTTP ${res.status}`;
				try {
					const data = await res.json();
					if (data && typeof data.error === 'string') message = data.error;
				} catch {
					// fall through
				}
				throw new Error(message);
			}
			const page = (await res.json()) as { items: CatalogLookupItem[] };
			results = page.items;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Search failed';
			results = [];
		} finally {
			searching = false;
		}
	}

	async function pick(item: CatalogLookupItem) {
		applyingId = item.id;
		try {
			await onpick(item);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to apply metadata';
			applyingId = null;
		}
	}

	function handleKey(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			e.preventDefault();
			void runSearch();
		}
	}
</script>

<Modal {open} maxWidth="max-w-3xl" {onclose}>
	<div class="flex flex-col gap-4">
		<header class="pr-8">
			<h2 class="text-lg font-semibold [overflow-wrap:anywhere]">
				Find metadata for "{firkinTitle}"
			</h2>
			<p class="mt-1 text-xs text-base-content/60">
				Searching <span class="badge badge-outline badge-xs">{addon}</span>. Pick a match to apply
				its title, year, description, and artwork — this rolls the firkin's version forward to a new
				CID.
			</p>
		</header>
		<div class="flex gap-2">
			<input
				type="search"
				class="input-bordered input input-sm flex-1"
				placeholder="Search…"
				bind:value={query}
				onkeydown={handleKey}
			/>
			<button
				class="btn btn-sm btn-primary"
				onclick={runSearch}
				disabled={searching || !query.trim()}
			>
				{searching ? 'Searching…' : 'Search'}
			</button>
		</div>
		{#if error}
			<div class="alert alert-error">
				<span>{error}</span>
			</div>
		{/if}
		{#if searching && results.length === 0}
			<p class="text-sm text-base-content/60">Searching…</p>
		{:else if !searching && results.length === 0 && !error}
			<p class="text-sm text-base-content/60">No results.</p>
		{:else}
			<div class="flex max-h-[60vh] flex-col gap-2 overflow-y-auto">
				{#each results as item (item.id)}
					<div
						class={classNames(
							'flex gap-3 rounded-box border border-base-content/10 bg-base-200 p-2',
							{ 'opacity-50': applyingId !== null && applyingId !== item.id }
						)}
					>
						{#if item.posterUrl}
							<img
								src={item.posterUrl}
								alt={item.title}
								class="h-24 w-16 shrink-0 rounded object-cover"
								loading="lazy"
							/>
						{:else}
							<div class="h-24 w-16 shrink-0 rounded bg-base-300"></div>
						{/if}
						<div class="flex flex-1 flex-col gap-1">
							<div class="flex items-baseline gap-2">
								<span class="font-semibold [overflow-wrap:anywhere]">{item.title}</span>
								{#if item.year !== null}
									<span class="text-xs text-base-content/60">{item.year}</span>
								{/if}
							</div>
							{#if item.description}
								<p class="line-clamp-3 text-xs text-base-content/70">{item.description}</p>
							{/if}
						</div>
						<button
							class="btn self-center btn-sm btn-primary"
							onclick={() => pick(item)}
							disabled={applyingId !== null}
						>
							{applyingId === item.id ? 'Applying…' : 'Use this'}
						</button>
					</div>
				{/each}
			</div>
		{/if}
	</div>
</Modal>
