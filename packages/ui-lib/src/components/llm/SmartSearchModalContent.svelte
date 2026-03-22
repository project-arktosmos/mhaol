<script lang="ts">
	import { onMount } from 'svelte';
	import { get } from 'svelte/store';
	import { llmService } from 'frontend/services/llm.service';
	import SmartSearchSection from './SmartSearchSection.svelte';

	const DEFAULT_MODEL = 'qwen2.5-1.5b-instruct-q4_k_m.gguf';

	const llmStore = llmService.store;

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

<div class="flex flex-col gap-4 p-4">
	<h2 class="text-lg font-bold">Smart Search</h2>
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
