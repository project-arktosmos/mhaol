<script lang="ts">
	import { llmService } from 'frontend/services/llm.service';
	import { recommendedModels } from 'frontend/data/recommended-models';
	import ModelManager from 'ui-lib/components/llm/ModelManager.svelte';

	const store = llmService.store;

	let status = $derived($store.status);
	let models = $derived($store.models);
	let downloadProgress = $derived($store.downloadProgress);
	let loading = $derived($store.loading);

	const filteredModels = recommendedModels.filter((m) => m.fileName.includes('1.5b'));

	$effect(() => {
		llmService.initialize();
	});
</script>

<div class="flex flex-col gap-4 p-4">
	<h2 class="text-lg font-bold">LLM Models</h2>
	<ModelManager
		{status}
		{models}
		{downloadProgress}
		{loading}
		recommendedModels={filteredModels}
		onLoadModel={(fileName) => llmService.loadModel(fileName)}
		onUnloadModel={() => llmService.unloadModel()}
		onDownloadModel={(repoId, fileName) => llmService.downloadModel(repoId, fileName)}
	/>
</div>
