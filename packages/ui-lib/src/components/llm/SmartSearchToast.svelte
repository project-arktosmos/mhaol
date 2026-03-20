<script lang="ts">
	import { onMount } from 'svelte';
	import { get } from 'svelte/store';
	import { llmService } from 'frontend/services/llm.service';
	import { smartSearchService } from 'frontend/services/smart-search.service';
	import SmartSearchSection from './SmartSearchSection.svelte';
	import type { SmartSearchTorrentResult } from 'frontend/types/smart-search.type';

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

{#if visible}
	<div class="toast toast-end toast-bottom z-50">
		<div class="h-[50vh] w-[40vw] overflow-y-auto rounded-lg border border-base-300 bg-base-300 p-4 shadow-lg">
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
				{onlibrarychange}
				{onstream}
			/>
		</div>
	</div>
{/if}
