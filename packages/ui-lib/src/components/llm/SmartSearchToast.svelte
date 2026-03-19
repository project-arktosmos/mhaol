<script lang="ts">
	import { onMount } from 'svelte';
	import { get } from 'svelte/store';
	import { llmService } from 'frontend/services/llm.service';
	import { smartSearchService } from 'frontend/services/smart-search.service';
	import SmartSearchSection from './SmartSearchSection.svelte';

	const REQUIRED_MODEL = {
		repoId: 'Qwen/Qwen2.5-1.5B-Instruct-GGUF',
		fileName: 'qwen2.5-1.5b-instruct-q4_k_m.gguf'
	};

	const llmStore = llmService.store;
	const searchStore = smartSearchService.store;

	let visible = $derived($searchStore.visible);

	onMount(async () => {
		await llmService.initialize();
		await ensureModel();
	});

	async function ensureModel() {
		const state = get(llmStore);

		// Already loaded and it's the right model
		if (state.status?.modelLoaded && state.status.currentModel === REQUIRED_MODEL.fileName) {
			return;
		}

		// Check if downloaded
		const downloaded = state.models.find((m) => m.fileName === REQUIRED_MODEL.fileName);

		if (!downloaded) {
			// Download it
			await llmService.downloadModel(REQUIRED_MODEL.repoId, REQUIRED_MODEL.fileName);
		}

		// Load it (re-read state after potential download)
		const updated = get(llmStore);
		const model = updated.models.find((m) => m.fileName === REQUIRED_MODEL.fileName);
		if (model && !model.isLoaded) {
			await llmService.loadModel(REQUIRED_MODEL.fileName);
		}
	}
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
			/>
		</div>
	</div>
{/if}
