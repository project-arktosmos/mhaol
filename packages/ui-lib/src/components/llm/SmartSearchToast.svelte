<script lang="ts">
	import { smartSearchService } from 'ui-lib/services/smart-search.service';
	import SmartSearchSection from './SmartSearchSection.svelte';
	import TvSmartSearchSection from './TvSmartSearchSection.svelte';
	import MusicSmartSearchSection from './MusicSmartSearchSection.svelte';
	import Modal from 'ui-lib/components/core/Modal.svelte';
	import type { SmartSearchTorrentResult } from 'ui-lib/types/smart-search.type';
	import { apiUrl } from 'ui-lib/lib/api-base';

	let {
		onlibrarychange,
		onstream
	}: {
		onlibrarychange?: () => void;
		onstream?: (candidate: SmartSearchTorrentResult) => void;
	} = $props();

	const searchStore = smartSearchService.store;

	// Auto-candidate logic runs even when modal is closed
	let selection = $derived($searchStore.selection);
	let mode = $derived(selection?.mode ?? null);
	let searching = $derived($searchStore.searching);
	let analyzing = $derived($searchStore.analyzing);
	let isMusic = $derived(selection?.type === 'music');

	const configStore = smartSearchService.configStore;

	let mediaConfig = $derived.by(() => {
		if (!selection) return null;
		const key =
			selection.type === 'movie'
				? 'movies'
				: selection.type === 'tv'
					? 'tv'
					: selection.type === 'music'
						? 'music'
						: selection.type === 'book'
							? 'books'
							: 'games';
		return $configStore[key];
	});

	let isTv = $derived(selection?.type === 'tv');

	let isMusicTabbed = $derived(isMusic && selection?.type === 'music' && selection?.musicSearchMode);

	let bestCandidate = $derived.by(() => {
		if (analyzing || searching) return null;

		// For TV, use the TV-specific best candidate logic
		if (isTv) {
			return smartSearchService.getBestTvCandidate();
		}

		// For music with tabbed search mode, use music-specific logic
		if (isMusicTabbed) {
			return smartSearchService.getBestMusicCandidate();
		}

		const raw = $searchStore.searchResults;
		const maxSE = Math.max(1, ...raw.map((r) => r.seeders));
		const maxLE = Math.max(1, ...raw.map((r) => r.leechers));
		const prefLang = (mediaConfig?.preferredLanguage ?? '').toLowerCase();
		const prefQuality = (mediaConfig?.preferredQuality ?? '').toLowerCase();
		const prefConsole = (mediaConfig?.preferredConsole ?? '').toLowerCase();
		const scored = raw
			.map((r) => {
				const sePct = Math.round((r.seeders / maxSE) * 100);
				const lePct = Math.round((r.leechers / maxLE) * 100);
				const relPct = r.analysis?.relevance ?? 0;
				const langBonus =
					prefLang && r.analysis && r.analysis.languages.toLowerCase().includes(prefLang) ? 100 : 0;
				const qualityBonus =
					prefQuality && r.analysis && r.analysis.quality.toLowerCase().includes(prefQuality)
						? 100
						: 0;
				const consoleBonus =
					prefConsole && r.analysis && r.analysis.reason.toLowerCase().includes('console matches')
						? 100
						: 0;
				return { r, score: sePct + lePct + relPct + langBonus + qualityBonus + consoleBonus };
			})
			.sort((a, b) => b.score - a.score);
		for (const { r } of scored) {
			if (!r.analysis) continue;
			if (r.analysis.relevance < 75) continue;
			return r;
		}
		return null;
	});

	let addingCandidate = $state(false);
	let candidateAdded = $state(false);

	$effect(() => {
		if (selection) {
			candidateAdded = false;
			addingCandidate = false;
		}
	});

	$effect(() => {
		if ($searchStore.pendingItemId) {
			onlibrarychange?.();
		}
	});

	async function handleAddCandidate() {
		if (!bestCandidate || !selection) return;
		addingCandidate = true;
		try {
			const configRes = await fetch(apiUrl('/api/torrent/config'));
			if (!configRes.ok) return;
			const config = await configRes.json();
			const basePath: string = config.downloadPath ?? '';
			if (!basePath) return;

			let subdir: string;
			switch (selection.type) {
				case 'music':
					subdir = 'music';
					break;
				case 'movie':
					subdir = 'movies';
					break;
				case 'game':
					subdir = 'games';
					break;
				case 'book':
					subdir = 'books';
					break;
				default:
					subdir = 'tv';
					break;
			}
			const downloadPath = `${basePath}/${subdir}`;

			const res = await fetch(apiUrl('/api/torrent/torrents'), {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					source: bestCandidate.magnetLink,
					downloadPath
				})
			});
			if (res.ok) {
				const torrentInfo = await res.json();
				candidateAdded = true;
				const outputPath: string = torrentInfo.outputPath ?? downloadPath;
				const infoHash: string = torrentInfo.infoHash ?? bestCandidate.infoHash;
				await smartSearchService.updateItemWithTorrent(infoHash, outputPath, 'download');
				onlibrarychange?.();
			}
		} catch {
			// ignore
		} finally {
			addingCandidate = false;
		}
	}

	$effect(() => {
		if (bestCandidate && !candidateAdded && !addingCandidate && mode) {
			if (mode === 'download') {
				handleAddCandidate();
			} else if (mode === 'stream') {
				onstream?.(bestCandidate);
				candidateAdded = true;
			} else if (mode === 'fetch') {
				smartSearchService.setFetchedCandidate(bestCandidate);
				candidateAdded = true;
			}
		}
	});
</script>

<Modal
	open={$searchStore.visible}
	maxWidth="max-w-[90vw]"
	onclose={() => smartSearchService.hide()}
>
	<h2 class="mb-3 text-sm font-semibold tracking-wide text-base-content/50 uppercase">
		Smart Search
	</h2>
	{#if isTv}
		<TvSmartSearchSection />
	{:else if isMusicTabbed}
		<MusicSmartSearchSection />
	{:else}
		<SmartSearchSection />
	{/if}
</Modal>
