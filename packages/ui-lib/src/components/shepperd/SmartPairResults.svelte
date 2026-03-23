<script lang="ts">
	import classNames from 'classnames';
	import type { SmartPairResult } from 'ui-lib/types/smart-pair.type';

	let {
		results,
		pairing = false,
		saving = false,
		saved = false,
		error = null,
		ontoggle,
		onacceptall,
		onrejectall,
		onsave,
		onreset
	}: {
		results: SmartPairResult[];
		pairing?: boolean;
		saving?: boolean;
		saved?: boolean;
		error?: string | null;
		ontoggle: (sourceId: string) => void;
		onacceptall: () => void;
		onrejectall: () => void;
		onsave: () => void;
		onreset: () => void;
	} = $props();

	const TMDB_IMAGE_BASE = 'https://image.tmdb.org/t/p';

	let acceptedCount = $derived(results.filter((r) => r.accepted).length);
	let matchedCount = $derived(results.filter((r) => r.matched).length);

	const confidenceBadge: Record<string, string> = {
		high: 'badge-success',
		medium: 'badge-warning',
		low: 'badge-error',
		none: 'badge-ghost'
	};

	const typeBadge: Record<string, string> = {
		movie: 'badge-primary',
		tv: 'badge-info'
	};

	const typeLabel: Record<string, string> = {
		movie: 'Movie',
		tv: 'TV'
	};
</script>

<div class="flex flex-col gap-4">
	{#if pairing}
		<div class="flex items-center gap-3 py-8">
			<span class="loading loading-md loading-spinner text-primary"></span>
			<span>Pairing items against TMDB...</span>
		</div>
	{:else if error}
		<div class="alert alert-error">
			<span>{error}</span>
			<button class="btn btn-ghost btn-sm" onclick={onreset}>Dismiss</button>
		</div>
	{:else if saved}
		<div class="alert alert-success">
			<span>Pinned items saved to library.</span>
			<button class="btn btn-ghost btn-sm" onclick={onreset}>Done</button>
		</div>
	{:else if results.length > 0}
		<div class="flex flex-wrap items-center gap-2">
			<span class="badge badge-primary">{matchedCount} of {results.length} matched</span>
			<span class="badge badge-secondary">{acceptedCount} accepted</span>
			<div class="flex-1"></div>
			<button class="btn btn-ghost btn-xs" onclick={onacceptall}>Accept All</button>
			<button class="btn btn-ghost btn-xs" onclick={onrejectall}>Reject All</button>
		</div>

		<div class="overflow-x-auto rounded-lg border border-base-300">
			<table class="table table-zebra table-sm">
				<thead>
					<tr>
						<th class="w-8"></th>
						<th>Source Title</th>
						<th></th>
						<th>TMDB Match</th>
						<th>Type</th>
						<th>Year</th>
						<th>Confidence</th>
						<th class="w-8"></th>
					</tr>
				</thead>
				<tbody>
					{#each results as result (result.sourceId)}
						<tr
							class={classNames({
								'opacity-40': !result.matched
							})}
						>
							<td>
								{#if result.tmdbPosterPath}
									<img
										src="{TMDB_IMAGE_BASE}/w92{result.tmdbPosterPath}"
										alt=""
										class="h-10 w-7 rounded object-cover"
									/>
								{/if}
							</td>
							<td class="font-medium">{result.sourceTitle}</td>
							<td class="text-base-content/40">→</td>
							<td>
								{#if result.tmdbTitle}
									<span class="font-medium">{result.tmdbTitle}</span>
								{:else}
									<span class="text-base-content/40 italic">No match</span>
								{/if}
							</td>
							<td>
								{#if result.tmdbType}
									<span class={classNames('badge badge-sm', typeBadge[result.tmdbType] ?? '')}>
										{typeLabel[result.tmdbType] ?? result.tmdbType}
									</span>
								{/if}
							</td>
							<td class="text-base-content/60">{result.tmdbYear ?? ''}</td>
							<td>
								<span
									class={classNames('badge badge-sm', confidenceBadge[result.confidence] ?? '')}
								>
									{result.confidence}
								</span>
							</td>
							<td>
								{#if result.matched}
									<input
										type="checkbox"
										class="checkbox checkbox-sm checkbox-primary"
										checked={result.accepted}
										onchange={() => ontoggle(result.sourceId)}
									/>
								{/if}
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>

		<div class="flex justify-end">
			<button
				class="btn btn-sm btn-primary"
				disabled={acceptedCount === 0 || saving}
				onclick={onsave}
			>
				{#if saving}
					<span class="loading loading-xs loading-spinner"></span>
				{/if}
				Save {acceptedCount} to Library
			</button>
		</div>
	{/if}
</div>
