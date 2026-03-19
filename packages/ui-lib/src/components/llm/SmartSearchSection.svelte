<script lang="ts">
	import classNames from 'classnames';
	import type { LlmStatus, LocalModel, LlmDownloadProgress } from 'frontend/types/llm.type';
	import { llmAdapter } from 'frontend/adapters/classes/llm.adapter';
	import { recommendedModels } from 'frontend/data/recommended-models';
	import { smartSearchService } from 'frontend/services/smart-search.service';
	import {
		formatSearchSize,
		formatSeeders,
		getSeedersColor,
		formatUploadDate
	} from 'frontend/utils/torrent-search/format';

	let {
		status,
		models,
		downloadProgress,
		loading,
		onLoadModel,
		onUnloadModel,
		onDownloadModel
	}: {
		status: LlmStatus | null;
		models: LocalModel[];
		downloadProgress: LlmDownloadProgress | null;
		loading: boolean;
		onLoadModel: (fileName: string) => void;
		onUnloadModel: () => void;
		onDownloadModel: (repoId: string, fileName: string) => void;
	} = $props();

	let downloadedFileNames = $derived(new Set(models.map((m) => m.fileName)));
	let showRecommended = $state(false);

	const searchStore = smartSearchService.store;
	let selection = $derived($searchStore.selection);
	let searching = $derived($searchStore.searching);
	let searchResults = $derived($searchStore.searchResults);
	let searchError = $derived($searchStore.searchError);

	let searchTerms = $derived.by(() => {
		if (!selection) return [];
		const { title, year, type } = selection;
		const typeLabel = type === 'movie' ? 'movie' : 'tv show';
		const parts: Array<{ term: string; components: string[] }> = [];

		parts.push({ term: title, components: ['title'] });
		parts.push({ term: `${title} ${year}`, components: ['title', 'year'] });
		parts.push({ term: `${title} ${typeLabel}`, components: ['title', 'type'] });
		parts.push({ term: `${title} ${year} ${typeLabel}`, components: ['title', 'year', 'type'] });

		return parts;
	});
</script>

{#if selection}
	<div class="mb-3 rounded bg-base-100 p-2">
		<div class="flex items-center justify-between">
			<div class="min-w-0 flex-1">
				<div class="truncate text-xs font-semibold">{selection.title}</div>
				<div class="flex items-center gap-1 text-xs text-base-content/50">
					<span>{selection.year}</span>
					<span class={classNames('badge badge-xs', {
						'badge-primary': selection.type === 'movie',
						'badge-info': selection.type === 'tv'
					})}>
						{selection.type === 'movie' ? 'Movie' : 'TV'}
					</span>
				</div>
			</div>
			<button
				class="btn btn-ghost btn-xs"
				onclick={() => smartSearchService.clear()}
			>
				&times;
			</button>
		</div>
	</div>

	{#if searchTerms.length > 0}
		<table class="table-xs table w-full">
			<thead>
				<tr>
					<th class="text-base-content/50">Search Term</th>
					<th class="text-base-content/50">Uses</th>
				</tr>
			</thead>
			<tbody>
				{#each searchTerms as { term, components }}
					<tr>
						<td class="font-mono text-xs">{term}</td>
						<td>
							<div class="flex gap-1">
								{#each components as comp}
									<span class="badge badge-outline badge-xs">{comp}</span>
								{/each}
							</div>
						</td>
					</tr>
				{/each}
			</tbody>
		</table>
	{/if}

	{#if searching}
		<div class="mt-3 flex items-center justify-center gap-2 py-4">
			<span class="loading loading-sm loading-spinner"></span>
			<span class="text-xs text-base-content/50">Searching torrents...</span>
		</div>
	{:else if searchError}
		<div class="mt-3 rounded bg-error/10 p-2 text-xs text-error">{searchError}</div>
	{:else if searchResults.length > 0}
		<div class="mt-3">
			<div class="mb-1 flex items-center justify-between">
				<span class="text-xs font-semibold text-base-content/50">
					{searchResults.length} result{searchResults.length !== 1 ? 's' : ''}
				</span>
			</div>
			<div class="overflow-x-auto">
				<table class="table-xs table w-full">
					<thead>
						<tr>
							<th>Name</th>
							<th class="text-right">Size</th>
							<th class="text-right">SE</th>
							<th class="text-right">LE</th>
							<th class="text-right">Uploaded</th>
						</tr>
					</thead>
					<tbody>
						{#each searchResults as result (result.infoHash)}
							<tr class="hover">
								<td class="max-w-xs">
									<div class="flex items-center gap-1">
										{#if result.isVip}
											<span class="badge badge-xs badge-warning" title="VIP">V</span>
										{:else if result.isTrusted}
											<span class="badge badge-xs badge-success" title="Trusted">T</span>
										{/if}
										<span class="truncate" title={result.name}>{result.name}</span>
									</div>
								</td>
								<td class="text-right text-nowrap">{formatSearchSize(result.size)}</td>
								<td class={classNames('text-right font-medium', getSeedersColor(result.seeders))}>
									{formatSeeders(result.seeders)}
								</td>
								<td class="text-right text-base-content/60">{formatSeeders(result.leechers)}</td>
								<td class="text-right text-nowrap text-base-content/60">
									{formatUploadDate(result.uploadedAt)}
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		</div>
	{/if}
{/if}

{#if status}
	<div class="mb-2 flex items-center gap-2 text-xs">
		<span
			class={classNames('h-1.5 w-1.5 rounded-full', {
				'bg-success': status.modelLoaded,
				'bg-base-300': !status.modelLoaded
			})}
		></span>
		<span class="truncate text-base-content/60">
			{status.modelLoaded ? status.currentModel : 'No model loaded'}
		</span>
	</div>
{/if}

{#if models.length > 0}
	<div class="flex flex-col gap-1">
		{#each models as model}
			<div class="flex items-center justify-between rounded bg-base-100 p-2">
				<div class="min-w-0 flex-1">
					<div class="truncate text-xs font-medium">{model.name}</div>
					<div class="text-xs text-base-content/40">
						{llmAdapter.formatModelSize(model.sizeBytes)}
					</div>
				</div>
				{#if model.isLoaded}
					<button class="btn btn-xs btn-warning" onclick={onUnloadModel}>Unload</button>
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
		{/each}
	</div>
{:else}
	<p class="text-xs text-base-content/40">No models downloaded</p>
{/if}

{#if downloadProgress}
	<div class="mt-2 rounded bg-base-100 p-2">
		<div class="mb-1 truncate text-xs font-medium">{downloadProgress.modelName}</div>
		<progress class="progress w-full progress-primary progress-sm" value={downloadProgress.percent} max="100"></progress>
		<p class="mt-0.5 text-xs text-base-content/40">
			{downloadProgress.percent.toFixed(1)}%
		</p>
	</div>
{/if}

<div class="mt-2">
	<button
		class="btn btn-ghost btn-xs w-full"
		onclick={() => (showRecommended = !showRecommended)}
	>
		{showRecommended ? 'Hide' : 'Download models'}
	</button>

	{#if showRecommended}
		<div class="mt-1 flex flex-col gap-1">
			{#each recommendedModels as model}
				{@const isDownloaded = downloadedFileNames.has(model.fileName)}
				<div class="flex items-center justify-between rounded bg-base-100 p-2">
					<div class="min-w-0 flex-1">
						<div class="truncate text-xs font-medium">{model.name}</div>
						<div class="text-xs text-base-content/40">{model.description}</div>
					</div>
					{#if isDownloaded}
						<span class="badge badge-xs badge-success">Downloaded</span>
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
			{/each}
		</div>
	{/if}
</div>
