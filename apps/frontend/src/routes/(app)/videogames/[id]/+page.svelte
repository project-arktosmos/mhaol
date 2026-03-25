<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { base } from '$app/paths';
	import { apiUrl } from 'ui-lib/lib/api-base';
	import { smartSearchService } from 'ui-lib/services/smart-search.service';
	import { torrentService } from 'ui-lib/services/torrent.service';
	import { gameExtendedToDisplay } from 'addons/retroachievements';
	import type { RaGameMetadata, RaGameExtended } from 'addons/retroachievements/types';
	import { CONSOLE_EJS_CORE, CONSOLE_ROM_EXTENSIONS } from 'addons/retroachievements/types';
	import type { TorrentInfo } from 'ui-lib/types/torrent.type';
	import GameDetailPage from 'ui-lib/components/videogames/GameDetailPage.svelte';

	let game = $state<RaGameMetadata | null>(null);
	let details = $state<RaGameMetadata | null>(null);
	let detailsLoading = $state(true);
	let fetchingGameId = $state<number | null>(null);
	let romFileUrl = $state<string | null>(null);
	let ejsCore = $state<string | null>(null);

	const searchStore = smartSearchService.store;
	const torrentState = torrentService.state;

	let id = $derived($page.params.id ?? '');
	let gameId = $derived(Number(id));

	let isFetching = $derived(
		fetchingGameId !== null &&
			fetchingGameId === gameId &&
			$searchStore.fetchedCandidate === null &&
			$searchStore.selection?.mode === 'fetch' &&
			$searchStore.searching
	);
	let isFetchedForCurrent = $derived(
		$searchStore.fetchedCandidate !== null && fetchingGameId === gameId
	);
	let fetchFailed = $derived(
		fetchingGameId !== null &&
			fetchingGameId === gameId &&
			!$searchStore.searching &&
			$searchStore.selection?.mode === 'fetch' &&
			$searchStore.fetchedCandidate === null
	);

	let currentFetchSteps = $derived.by(() => {
		if (!isFetching && !isFetchedForCurrent && !fetchFailed) return null;
		if (isFetchedForCurrent) {
			return { terms: true, search: true, searching: false, eval: true, done: true };
		}
		const s = $searchStore;
		const hasResults = s.searchResults.length > 0;
		const hasAnalysis = s.searchResults.some((r) => r.analysis !== null);
		return {
			terms: s.selection !== null,
			search: !s.searching && hasResults,
			searching: s.searching,
			eval: hasAnalysis,
			done: s.fetchedCandidate !== null
		};
	});

	let matchedTorrent = $derived.by((): TorrentInfo | null => {
		const candidate = $searchStore.fetchedCandidate;
		if (candidate?.infoHash) {
			const t = torrentService.findByHash(candidate.infoHash);
			if (t) return t;
		}
		return null;
	});

	let currentTorrentStatus = $derived(matchedTorrent);
	let isDownloaded = $derived(currentTorrentStatus?.state === 'seeding');

	$effect(() => {
		if (!isDownloaded || !game) return;
		const candidate = $searchStore.fetchedCandidate;
		if (!candidate?.infoHash) return;

		const consoleId = game.consoleId;
		const core = CONSOLE_EJS_CORE[consoleId];
		if (!core) {
			ejsCore = null;
			romFileUrl = null;
			return;
		}

		const validExts = CONSOLE_ROM_EXTENSIONS[consoleId] ?? [];
		const infoHash = candidate.infoHash;

		fetch(apiUrl(`/api/torrent/torrents/${infoHash}/files`))
			.then((res) => (res.ok ? res.json() : []))
			.then((files: Array<{ id: number; name: string; size: number }>) => {
				const romFile = files.find((f) => {
					const ext = f.name.split('.').pop()?.toLowerCase() ?? '';
					return validExts.includes(ext);
				});
				if (romFile) {
					romFileUrl = apiUrl(`/api/torrent/torrents/${infoHash}/serve/${romFile.id}`);
					ejsCore = core;
				} else {
					romFileUrl = null;
					ejsCore = null;
				}
			});
	});

	async function fetchGame(id: string) {
		detailsLoading = true;
		smartSearchService.clear();
		romFileUrl = null;
		ejsCore = null;
		try {
			const res = await fetch(apiUrl(`/api/retroachievements/games/${id}`));
			if (!res.ok) throw new Error('Failed to fetch game');
			const data = await res.json();
			const detail = gameExtendedToDisplay(data as RaGameExtended);
			game = detail;
			details = detail;
		} catch {
			game = null;
			details = null;
		}
		detailsLoading = false;
	}

	function handleFetch() {
		if (!game) return;
		fetchingGameId = game.id;
		smartSearchService.select({
			title: game.title,
			year: '',
			type: 'game',
			retroachievementsId: game.id,
			consoleId: game.consoleId,
			consoleName: game.consoleName,
			mode: 'fetch'
		});
	}

	function handleDownload() {
		const candidate = smartSearchService.getFetchedCandidate();
		if (!candidate) return;
		smartSearchService.startDownload(candidate);
	}

	onMount(() => {
		fetchGame(id);
	});
</script>

{#if game}
	<GameDetailPage
		{game}
		{details}
		{detailsLoading}
		fetching={isFetching}
		fetched={isFetchedForCurrent}
		fetchSteps={currentFetchSteps}
		torrentStatus={currentTorrentStatus}
		fetchedTorrent={$searchStore.fetchedCandidate
			? {
					name: $searchStore.fetchedCandidate.name,
					quality: $searchStore.fetchedCandidate.analysis?.quality ?? '',
					languages: $searchStore.fetchedCandidate.analysis?.languages ?? ''
				}
			: null}
		{fetchFailed}
		{romFileUrl}
		{ejsCore}
		onfetch={handleFetch}
		ondownload={handleDownload}
		onshowsearch={() => smartSearchService.show()}
		onback={() => goto(`${base}/videogames`)}
	/>
{:else if detailsLoading}
	<div class="flex flex-1 items-center justify-center">
		<span class="loading loading-lg loading-spinner"></span>
	</div>
{:else}
	<div class="flex flex-1 flex-col items-center justify-center gap-2">
		<p class="text-sm opacity-60">Game not found</p>
		<button class="btn btn-ghost btn-sm" onclick={() => goto(`${base}/videogames`)}>Back to games</button>
	</div>
{/if}
