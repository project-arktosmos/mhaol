<script lang="ts">
	import { onMount } from 'svelte';
	import { get } from 'svelte/store';
	import { llmService } from 'ui-lib/services/llm.service';
	import { smartSearchService } from 'ui-lib/services/smart-search.service';
	import SmartSearchSection from './SmartSearchSection.svelte';
	import Modal from 'ui-lib/components/core/Modal.svelte';
	import type { SmartSearchTorrentResult } from 'ui-lib/types/smart-search.type';
	import { apiUrl } from 'ui-lib/lib/api-base';

	const DEFAULT_MODEL = 'qwen2.5-1.5b-instruct-q4_k_m.gguf';

	let {
		onlibrarychange,
		onstream
	}: {
		onlibrarychange?: () => void;
		onstream?: (candidate: SmartSearchTorrentResult) => void;
	} = $props();

	const llmStore = llmService.store;
	const searchStore = smartSearchService.store;

	let visible = $derived($searchStore.visible);

	// Auto-candidate logic runs even when modal is closed
	let selection = $derived($searchStore.selection);
	let mode = $derived(selection?.mode ?? null);
	let searching = $derived($searchStore.searching);
	let analyzing = $derived($searchStore.analyzing);
	let isMusic = $derived(selection?.type === 'music');

	let bestCandidate = $derived.by(() => {
		if (analyzing || searching) return null;
		const results = [...$searchStore.searchResults].sort((a, b) => {
			if (b.seeders !== a.seeders) return b.seeders - a.seeders;
			if (b.leechers !== a.leechers) return b.leechers - a.leechers;
			const relA = a.analysis?.relevance ?? -1;
			const relB = b.analysis?.relevance ?? -1;
			return relB - relA;
		});
		const prefLang = 'english';
		const prefQuality = isMusic ? 'flac' : '1080p';
		for (const r of results) {
			if (!r.analysis) continue;
			if (r.analysis.relevance < 75) continue;
			if (!isMusic) {
				if (!r.analysis.languages.toLowerCase().includes(prefLang)) continue;
				if (!r.analysis.quality.toLowerCase().includes(prefQuality)) continue;
			}
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

			const subdir =
				selection.type === 'music' ? 'music' : selection.type === 'movie' ? 'movies' : 'tv';
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

	onMount(async () => {
		await llmService.initialize();
		const state = get(llmStore);
		if (state.status?.modelLoaded) return;
		const model = state.models.find((m) => m.fileName === DEFAULT_MODEL);
		if (model && !model.isLoaded) {
			await llmService.loadModel(DEFAULT_MODEL);
		}
	});
</script>

<Modal open={visible} maxWidth="max-w-3xl" onclose={() => smartSearchService.hide()}>
	<h2 class="mb-3 text-sm font-semibold tracking-wide text-base-content/50 uppercase">
		Smart Search
	</h2>
	<SmartSearchSection
		status={$llmStore.status}
		models={$llmStore.models}
		downloadProgress={$llmStore.downloadProgress}
		loading={$llmStore.loading}
		onLoadModel={(fileName) => llmService.loadModel(fileName)}
		onUnloadModel={() => llmService.unloadModel()}
		onDownloadModel={(repoId, fileName) => llmService.downloadModel(repoId, fileName)}
	/>
</Modal>
