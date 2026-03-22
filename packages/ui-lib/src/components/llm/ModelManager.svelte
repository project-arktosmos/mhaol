<script lang="ts">
	import classNames from 'classnames';
	import type {
		LlmStatus,
		LocalModel,
		LlmDownloadProgress,
		RecommendedModel
	} from 'ui-lib/types/llm.type';
	import { llmAdapter } from 'ui-lib/adapters/classes/llm.adapter';
	import { recommendedModels as allRecommendedModels } from 'ui-lib/data/recommended-models';

	let {
		status,
		models,
		downloadProgress,
		loading,
		onLoadModel,
		onUnloadModel,
		onDownloadModel,
		recommendedModels = allRecommendedModels
	}: {
		status: LlmStatus | null;
		models: LocalModel[];
		downloadProgress: LlmDownloadProgress | null;
		loading: boolean;
		onLoadModel: (fileName: string) => void;
		onUnloadModel: () => void;
		onDownloadModel: (repoId: string, fileName: string) => void;
		recommendedModels?: RecommendedModel[];
	} = $props();

	let downloadedFileNames = $derived(new Set(models.map((m) => m.fileName)));
</script>

<div class="space-y-6">
	{#if status}
		<div class="rounded-lg bg-base-200 p-4">
			<h3 class="mb-2 text-sm font-semibold">Engine Status</h3>
			<div class="grid grid-cols-2 gap-2 text-sm">
				<span class="text-base-content/60">Status</span>
				<span>{status.modelLoaded ? 'Model Loaded' : 'No Model'}</span>
				{#if status.currentModel}
					<span class="text-base-content/60">Model</span>
					<span class="truncate">{status.currentModel}</span>
				{/if}
				<span class="text-base-content/60">Models Dir</span>
				<span class="truncate text-xs">{status.modelsDir}</span>
			</div>
		</div>
	{/if}

	<div>
		<h3 class="mb-2 text-sm font-semibold">Downloaded Models</h3>
		{#if models.length === 0}
			<p class="text-sm text-base-content/40">No models downloaded yet</p>
		{:else}
			<div class="space-y-2">
				{#each models as model}
					<div
						class={classNames(
							'flex items-center justify-between rounded-lg border border-base-300 p-3',
							{ 'border-primary': model.isLoaded }
						)}
					>
						<div>
							<div class="text-sm font-medium">{model.name}</div>
							<div class="text-xs text-base-content/50">
								{llmAdapter.formatModelSize(model.sizeBytes)}
							</div>
						</div>
						<div>
							{#if model.isLoaded}
								<button class="btn btn-xs btn-warning" onclick={onUnloadModel}> Unload </button>
							{:else}
								<button
									class="btn btn-xs btn-primary"
									onclick={() => onLoadModel(model.fileName)}
									disabled={loading}
								>
									{#if loading}
										<span class="loading loading-xs loading-spinner"></span>
									{:else}
										Load
									{/if}
								</button>
							{/if}
						</div>
					</div>
				{/each}
			</div>
		{/if}
	</div>

	{#if downloadProgress}
		<div class="rounded-lg bg-base-200 p-4">
			<h3 class="mb-2 text-sm font-semibold">Downloading: {downloadProgress.modelName}</h3>
			<progress class="progress w-full progress-primary" value={downloadProgress.percent} max="100"
			></progress>
			<p class="mt-1 text-xs text-base-content/50">
				{downloadProgress.percent.toFixed(1)}% — {llmAdapter.formatModelSize(
					downloadProgress.downloadedBytes
				)} / {llmAdapter.formatModelSize(downloadProgress.totalBytes)}
			</p>
		</div>
	{/if}

	<div>
		<h3 class="mb-2 text-sm font-semibold">Recommended Models</h3>
		<div class="space-y-2">
			{#each recommendedModels as model}
				{@const isDownloaded = downloadedFileNames.has(model.fileName)}
				<div class="flex items-center justify-between rounded-lg border border-base-300 p-3">
					<div>
						<div class="text-sm font-medium">{model.name}</div>
						<div class="text-xs text-base-content/50">{model.description}</div>
					</div>
					<div>
						{#if isDownloaded}
							<span class="badge badge-sm badge-success">Downloaded</span>
						{:else}
							<button
								class="btn btn-xs btn-secondary"
								onclick={() => onDownloadModel(model.repoId, model.fileName)}
								disabled={downloadProgress !== null}
							>
								Download
							</button>
						{/if}
					</div>
				</div>
			{/each}
		</div>
	</div>
</div>
