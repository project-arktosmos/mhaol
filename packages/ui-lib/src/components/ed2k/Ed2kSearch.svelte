<script lang="ts">
	import classNames from 'classnames';
	import { ed2kService } from 'ui-lib/services/ed2k.service';
	import type { Ed2kSearchResult } from 'ui-lib/types/ed2k.type';

	const serviceState = ed2kService.state;

	let query = $state('');
	let addingHashes = $state<Record<string, boolean>>({});

	function formatSize(bytes: number): string {
		if (bytes <= 0) return '0 B';
		const units = ['B', 'KB', 'MB', 'GB', 'TB'];
		const i = Math.floor(Math.log(bytes) / Math.log(1024));
		const v = bytes / Math.pow(1024, i);
		return `${v.toFixed(i === 0 ? 0 : 1)} ${units[i]}`;
	}

	async function handleSearch() {
		const trimmed = query.trim();
		if (!trimmed) return;
		await ed2kService.search(trimmed);
	}

	async function handleAdd(result: Ed2kSearchResult) {
		addingHashes = { ...addingHashes, [result.fileHash]: true };
		await ed2kService.addFile(result.ed2kLink);
		const next = { ...addingHashes };
		delete next[result.fileHash];
		addingHashes = next;
	}

	function handleClear() {
		ed2kService.clearSearch();
		query = '';
	}
</script>

<div class="flex flex-col gap-3">
	<div class="join w-full">
		<input
			type="text"
			bind:value={query}
			onkeydown={(e) => e.key === 'Enter' && handleSearch()}
			placeholder="Search the ed2k network…"
			class="input-bordered input join-item flex-1"
		/>
		<button
			class="btn join-item btn-primary"
			onclick={handleSearch}
			disabled={!query.trim() || $serviceState.searching}
		>
			{#if $serviceState.searching}
				<span class="loading loading-sm loading-spinner"></span>
			{:else}
				Search
			{/if}
		</button>
		{#if $serviceState.searchResults.length > 0}
			<button class="btn join-item btn-ghost" onclick={handleClear}>Clear</button>
		{/if}
	</div>

	{#if !$serviceState.searching && $serviceState.searchResults.length === 0 && $serviceState.searchQuery}
		<div class="alert-sm alert">
			<span>No results for "{$serviceState.searchQuery}".</span>
		</div>
	{/if}

	{#if $serviceState.searchResults.length > 0}
		<div class="overflow-x-auto">
			<table class="table table-zebra table-sm">
				<thead>
					<tr>
						<th>Name</th>
						<th class="text-right">Size</th>
						<th class="text-right">Sources</th>
						<th class="text-right">Complete</th>
						<th></th>
					</tr>
				</thead>
				<tbody>
					{#each $serviceState.searchResults as result (result.fileHash)}
						<tr>
							<td class="max-w-md">
								<div class="truncate" title={result.name}>{result.name}</div>
								<div class="font-mono text-xs text-base-content/40">
									{result.fileHash}
								</div>
							</td>
							<td class="text-right">{formatSize(result.size)}</td>
							<td class="text-right">{result.sources}</td>
							<td class="text-right">{result.completeSources}</td>
							<td class="text-right">
								<button
									class={classNames('btn btn-xs btn-primary')}
									onclick={() => handleAdd(result)}
									disabled={addingHashes[result.fileHash] || !$serviceState.initialized}
								>
									{#if addingHashes[result.fileHash]}
										<span class="loading loading-xs loading-spinner"></span>
									{:else}
										Add
									{/if}
								</button>
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{/if}
</div>
