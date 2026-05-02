<script lang="ts">
	import { onMount, untrack } from 'svelte';
	import { base } from '$app/paths';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { listSources, type CatalogSource } from '$lib/catalog.service';

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

	const rootPath = `${base}/`;

	const activeAddon = $derived.by(() => {
		const fromUrl = page.url.searchParams.get('addon') ?? '';
		if (sources.length === 0) return fromUrl;
		if (fromUrl && sources.some((s) => s.id === fromUrl)) return fromUrl;
		return sources[0]?.id ?? '';
	});

	const currentSource = $derived(sources.find((s) => s.id === activeAddon));
	const showSearchFieldSelect = $derived(activeAddon === 'musicbrainz');

	let query = $state(page.url.searchParams.get('q') ?? '');
	let searchField = $state<'artist' | 'release'>(
		(page.url.searchParams.get('field') as 'artist' | 'release') ?? 'artist'
	);

	$effect(() => {
		const urlQ = page.url.searchParams.get('q') ?? '';
		untrack(() => {
			if (urlQ !== query) query = urlQ;
		});
	});
	$effect(() => {
		const urlField = (page.url.searchParams.get('field') as 'artist' | 'release') ?? 'artist';
		untrack(() => {
			if (urlField !== searchField) searchField = urlField;
		});
	});

	let debounceTimer: ReturnType<typeof setTimeout> | null = null;

	function syncToUrl() {
		const url = new URL(page.url);
		const trimmed = query.trim();
		if (trimmed) url.searchParams.set('q', trimmed);
		else url.searchParams.delete('q');
		if (activeAddon === 'musicbrainz' && searchField !== 'artist') {
			url.searchParams.set('field', searchField);
		} else {
			url.searchParams.delete('field');
		}
		void goto(`${rootPath}${url.search}`, {
			keepFocus: true,
			noScroll: true,
			replaceState: true
		});
	}

	function scheduleSync() {
		if (debounceTimer) clearTimeout(debounceTimer);
		debounceTimer = setTimeout(syncToUrl, 300);
	}

	function onFieldChange() {
		syncToUrl();
	}
</script>

<div class="flex items-center gap-2">
	{#if showSearchFieldSelect}
		<select
			class="select-bordered select w-32 select-sm"
			bind:value={searchField}
			onchange={onFieldChange}
			title="Which release-group field to search on"
		>
			<option value="artist">Artist name</option>
			<option value="release">Album title</option>
		</select>
	{/if}
	<input
		type="search"
		class="input-bordered input input-sm w-64"
		placeholder={activeAddon
			? `Search ${currentSource?.label ?? activeAddon}…`
			: 'Pick an addon to search'}
		disabled={!activeAddon}
		bind:value={query}
		oninput={scheduleSync}
	/>
</div>
