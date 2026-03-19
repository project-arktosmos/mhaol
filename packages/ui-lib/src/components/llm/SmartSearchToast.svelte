<script lang="ts">
	import { onMount } from 'svelte';
	import { get } from 'svelte/store';
	import { llmService } from 'frontend/services/llm.service';
	import { smartSearchService } from 'frontend/services/smart-search.service';
	import SmartSearchSection from './SmartSearchSection.svelte';

	const llmStore = llmService.store;
	const searchStore = smartSearchService.store;

	let visible = $derived($searchStore.visible);

	onMount(async () => {
		await llmService.initialize();

		const state = get(llmStore);
		const { status, models } = state;

		if (status?.modelLoaded) return;

		if (models.length === 1) {
			llmService.loadModel(models[0].fileName);
		} else if (models.length > 1 && status?.currentModel) {
			const match = models.find((m) => m.fileName === status.currentModel);
			if (match) llmService.loadModel(match.fileName);
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
			/>
		</div>
	</div>
{/if}
