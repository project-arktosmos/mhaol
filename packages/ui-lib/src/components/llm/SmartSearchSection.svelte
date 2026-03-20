<script lang="ts">
	import classNames from 'classnames';
	import type { LlmStatus, LocalModel, LlmDownloadProgress } from 'frontend/types/llm.type';
	import { llmAdapter } from 'frontend/adapters/classes/llm.adapter';
	import { recommendedModels } from 'frontend/data/recommended-models';
	import { smartSearchService } from 'frontend/services/smart-search.service';
	import { apiUrl } from 'frontend/lib/api-base';
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
		onDownloadModel,
		onlibrarychange
	}: {
		status: LlmStatus | null;
		models: LocalModel[];
		downloadProgress: LlmDownloadProgress | null;
		loading: boolean;
		onLoadModel: (fileName: string) => void;
		onUnloadModel: () => void;
		onDownloadModel: (repoId: string, fileName: string) => void;
		onlibrarychange?: () => void;
	} = $props();

	let downloadedFileNames = $derived(new Set(models.map((m) => m.fileName)));
	let showRecommended = $state(false);

	let preferredLanguage = $state('English');
	let preferredQuality = $state('1080p');

	const languages = ['English', 'Spanish', 'French', 'German', 'Italian', 'Portuguese', 'Russian', 'Japanese', 'Korean', 'Chinese', 'Hindi', 'Arabic', 'Dutch', 'Swedish', 'Norwegian', 'Danish', 'Finnish', 'Polish', 'Turkish', 'Thai'];
	const qualities = ['4K', '2160p', '1080p', '720p', '480p'];

	const searchStore = smartSearchService.store;
	let selection = $derived($searchStore.selection);
	let searching = $derived($searchStore.searching);
	let analyzing = $derived($searchStore.analyzing);
	let searchResults = $derived(
		[...$searchStore.searchResults].sort((a, b) => {
			const matchA = a.analysis
				? (a.analysis.languages.toLowerCase().includes(preferredLanguage.toLowerCase()) ? 1 : 0)
				+ (a.analysis.quality.toLowerCase().includes(preferredQuality.toLowerCase()) ? 1 : 0)
				: -1;
			const matchB = b.analysis
				? (b.analysis.languages.toLowerCase().includes(preferredLanguage.toLowerCase()) ? 1 : 0)
				+ (b.analysis.quality.toLowerCase().includes(preferredQuality.toLowerCase()) ? 1 : 0)
				: -1;
			if (matchB !== matchA) return matchB - matchA;
			if (b.seeders !== a.seeders) return b.seeders - a.seeders;
			if (b.leechers !== a.leechers) return b.leechers - a.leechers;
			const relA = a.analysis?.relevance ?? -1;
			const relB = b.analysis?.relevance ?? -1;
			return relB - relA;
		})
	);
	let searchError = $derived($searchStore.searchError);

	$effect(() => {
		if (selection) {
			candidateAdded = false;
			addingCandidate = false;
		}
	});

	let bestCandidate = $derived.by(() => {
		if (analyzing || searching) return null;
		for (const r of searchResults) {
			if (!r.analysis) continue;
			if (r.analysis.relevance < 75) continue;
			if (!r.analysis.languages.toLowerCase().includes(preferredLanguage.toLowerCase())) continue;
			if (!r.analysis.quality.toLowerCase().includes(preferredQuality.toLowerCase())) continue;
			return r;
		}
		return null;
	});

	let addingCandidate = $state(false);
	let candidateAdded = $state(false);

	$effect(() => {
		if (bestCandidate && !candidateAdded && !addingCandidate) {
			handleAddCandidate();
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

			const subdir = selection.type === 'movie' ? 'movies' : 'tv';
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
				await addLibraryItem(selection, outputPath, basePath);
			}
		} catch {
			// ignore
		} finally {
			addingCandidate = false;
		}
	}

	async function addLibraryItem(
		sel: NonNullable<typeof selection>,
		outputPath: string,
		basePath: string
	) {
		try {
			const subdir = sel.type === 'movie' ? 'movies' : 'tv';
			const targetPath = `${basePath}/${subdir}`;

			// Find the library whose path matches the type-specific subdir
			const libRes = await fetch(apiUrl('/api/libraries'));
			if (!libRes.ok) return;
			const libraries: Array<{ id: string; path: string; libraryType: string }> =
				await libRes.json();
			let library = libraries.find((l) => l.path === targetPath);

			// Create library if it doesn't exist
			if (!library) {
				const createRes = await fetch(apiUrl('/api/libraries'), {
					method: 'POST',
					headers: { 'Content-Type': 'application/json' },
					body: JSON.stringify({
						name: sel.type === 'movie' ? 'Movies' : 'TV Shows',
						path: targetPath,
						libraryType: subdir
					})
				});
				if (!createRes.ok) return;
				library = await createRes.json();
			}
			if (!library) return;

			const itemRes = await fetch(apiUrl(`/api/libraries/${library.id}/items`), {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					name: sel.title,
					path: outputPath,
					mediaType: 'video',
					categoryId: subdir,
					tmdbId: sel.tmdbId
				})
			});
			if (itemRes.ok) {
				onlibrarychange?.();
			}
		} catch {
			// best-effort
		}
	}

	let searchTerms = $derived.by(() => {
		if (!selection) return [];
		const { title, year } = selection;
		return [
			{ term: title, components: ['title'] },
			{ term: `${title} ${year}`, components: ['title', 'year'] }
		];
	});

	let stepTerms = $derived(selection !== null);
	let stepSearch = $derived(stepTerms && !searching && searchResults.length > 0);
	let stepEval = $derived(stepSearch && searchResults.some((r) => r.analysis !== null));
	let stepDone = $derived(stepEval && candidateAdded);
</script>

<ul class="steps steps-horizontal mb-4 w-full text-xs">
	<li class={classNames('step', { 'step-success': stepTerms })}>Terms</li>
	<li class={classNames('step', { 'step-success': stepSearch })}>{searching ? 'Searching...' : 'Search'}</li>
	<li class={classNames('step', { 'step-success': stepEval })}>Analysis</li>
	<li class={classNames('step', { 'step-success': stepDone })}>{bestCandidate && !candidateAdded ? 'Ready' : candidateAdded ? 'Done' : 'Candidate'}</li>
</ul>

<div class="mb-3 flex items-center gap-2">
	<select class="select-bordered select select-xs" bind:value={preferredLanguage}>
		{#each languages as lang}
			<option value={lang}>{lang}</option>
		{/each}
	</select>
	<select class="select-bordered select select-xs" bind:value={preferredQuality}>
		{#each qualities as q}
			<option value={q}>{q}</option>
		{/each}
	</select>
</div>

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
					{#if analyzing}
						<span class="loading loading-xs loading-spinner ml-1"></span>
					{/if}
				</span>
			</div>
			<div class="overflow-x-auto">
				<table class="table-xs table w-full">
					<thead>
						<tr>
							<th>Name</th>
							<th>Query</th>
							<th class="text-right">Size</th>
							<th class="text-right">SE</th>
							<th class="text-right">LE</th>
							<th class="text-right">Uploaded</th>
							<th>Quality</th>
							<th>Lang</th>
							<th>Subs</th>
							<th class="text-right">Rel%</th>
							<th>Reason</th>
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
								<td>
									<div class="flex flex-col gap-0.5">
										{#each result.searchQueries as q}
											<span class="truncate text-xs text-base-content/40" title={q}>{q}</span>
										{/each}
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
								{#if result.analyzing}
									<td colspan="5" class="text-center">
										<span class="loading loading-xs loading-spinner"></span>
									</td>
								{:else if result.analysis}
									<td class="text-nowrap text-xs">{result.analysis.quality}</td>
									<td class="text-nowrap text-xs">{result.analysis.languages}</td>
									<td class="text-nowrap text-xs">{result.analysis.subs}</td>
									<td class={classNames('text-right text-xs font-medium', {
										'text-success': result.analysis.relevance >= 80,
										'text-warning': result.analysis.relevance >= 50 && result.analysis.relevance < 80,
										'text-error': result.analysis.relevance < 50
									})}>
										{result.analysis.relevance}%
									</td>
									<td class="max-w-xs text-xs text-base-content/60" title={result.analysis.reason}>
										<span class="line-clamp-2">{result.analysis.reason}</span>
									</td>
								{:else}
									<td colspan="5"></td>
								{/if}
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		</div>
	{/if}

	{#if bestCandidate}
		<div class="mt-3 rounded-lg border border-success/50 bg-success/10 p-3">
			<div class="mb-2 flex items-center gap-2">
				<span class="badge badge-sm badge-success">Best Match</span>
				<span class="text-xs font-semibold">{bestCandidate.analysis?.relevance}% relevance</span>
			</div>
			<div class="mb-2 text-xs font-medium" title={bestCandidate.name}>
				{bestCandidate.name}
			</div>
			<div class="mb-2 flex flex-wrap gap-2 text-xs text-base-content/60">
				<span>{bestCandidate.analysis?.quality}</span>
				<span>{bestCandidate.analysis?.languages}</span>
				{#if bestCandidate.analysis?.subs && bestCandidate.analysis.subs !== 'none'}
					<span>Subs: {bestCandidate.analysis.subs}</span>
				{/if}
				<span>{formatSearchSize(bestCandidate.size)}</span>
				<span class={getSeedersColor(bestCandidate.seeders)}>{formatSeeders(bestCandidate.seeders)} SE</span>
			</div>
			<div class="mb-2 text-xs text-base-content/50">{bestCandidate.analysis?.reason}</div>
			{#if candidateAdded}
				<span class="badge badge-sm badge-success">Added to downloads</span>
			{:else}
				<button
					class="btn btn-sm btn-success"
					onclick={handleAddCandidate}
					disabled={addingCandidate}
				>
					{#if addingCandidate}
						<span class="loading loading-xs loading-spinner"></span>
					{:else}
						Download & Add to Library
					{/if}
				</button>
			{/if}
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
