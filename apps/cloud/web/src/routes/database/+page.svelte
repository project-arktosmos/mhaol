<script lang="ts">
	import { onMount } from 'svelte';
	import classNames from 'classnames';
	import { databaseService } from '$lib/database.service';

	const tablesStore = databaseService.state;
	const recordsStore = databaseService.records;

	let pageSize = $state(100);

	onMount(() => {
		databaseService.refresh();
	});

	function selectTable(name: string) {
		databaseService.loadTable(name, pageSize, 0);
	}

	function nextPage() {
		const next = $recordsStore.offset + $recordsStore.limit;
		if (next >= $recordsStore.total) return;
		if ($recordsStore.table) {
			databaseService.loadTable($recordsStore.table, $recordsStore.limit, next);
		}
	}

	function prevPage() {
		const prev = Math.max(0, $recordsStore.offset - $recordsStore.limit);
		if (prev === $recordsStore.offset) return;
		if ($recordsStore.table) {
			databaseService.loadTable($recordsStore.table, $recordsStore.limit, prev);
		}
	}

	function refreshCurrent() {
		if ($recordsStore.table) {
			databaseService.loadTable($recordsStore.table, $recordsStore.limit, $recordsStore.offset);
		}
	}

	function formatRecord(record: unknown): string {
		try {
			return JSON.stringify(record, null, 2);
		} catch {
			return String(record);
		}
	}

	function recordKey(record: unknown, index: number): string {
		if (record && typeof record === 'object' && 'id' in record) {
			const id = (record as { id: unknown }).id;
			if (typeof id === 'string') return id;
			if (id && typeof id === 'object') return JSON.stringify(id);
		}
		return String(index);
	}
</script>

<svelte:head>
	<title>Mhaol Cloud — Database</title>
</svelte:head>

<div class="flex min-h-full min-w-0 flex-1 flex-col gap-4 p-6">
	<header class="flex items-start justify-between gap-4">
		<div>
			<h1 class="text-2xl font-bold">Database</h1>
			<p class="text-sm text-base-content/60">
				{#if $tablesStore.namespace}
					Explorer for the <span class="font-mono">{$tablesStore.namespace}</span> /
					<span class="font-mono">{$tablesStore.database}</span> SurrealDB store.
				{:else}
					Explorer for the cloud SurrealDB store.
				{/if}
			</p>
		</div>
		<div class="flex items-center gap-2">
			<label class="label gap-2 text-xs">
				<span class="label-text">Page size</span>
				<select
					class="select-bordered select select-sm"
					bind:value={pageSize}
					onchange={() => {
						if ($recordsStore.table) databaseService.loadTable($recordsStore.table, pageSize, 0);
					}}
				>
					<option value={25}>25</option>
					<option value={50}>50</option>
					<option value={100}>100</option>
					<option value={250}>250</option>
					<option value={500}>500</option>
				</select>
			</label>
			<button
				class="btn btn-outline btn-sm"
				onclick={() => databaseService.refresh()}
				disabled={$tablesStore.loading}
			>
				{$tablesStore.loading ? 'Refreshing…' : 'Refresh tables'}
			</button>
		</div>
	</header>

	{#if $tablesStore.error}
		<div class="alert alert-error">
			<span>{$tablesStore.error}</span>
		</div>
	{/if}

	<div class="flex min-h-0 flex-1 gap-4">
		<aside
			class="w-64 shrink-0 overflow-y-auto rounded-box border border-base-content/10 bg-base-200"
		>
			<div
				class="border-b border-base-content/10 px-3 py-2 text-xs font-semibold text-base-content/60 uppercase"
			>
				Tables ({$tablesStore.tables.length})
			</div>
			{#if $tablesStore.tables.length === 0 && !$tablesStore.loading}
				<p class="p-3 text-sm text-base-content/60">No tables yet.</p>
			{:else}
				<ul class="menu w-full menu-sm">
					{#each $tablesStore.tables as table (table.name)}
						<li>
							<button
								type="button"
								onclick={() => selectTable(table.name)}
								class={classNames('flex justify-between gap-2', {
									active: $recordsStore.table === table.name
								})}
							>
								<span class="truncate font-mono">{table.name}</span>
								<span class="badge badge-ghost badge-sm">{table.record_count}</span>
							</button>
						</li>
					{/each}
				</ul>
			{/if}
		</aside>

		<section class="flex min-w-0 flex-1 flex-col gap-3">
			{#if !$recordsStore.table}
				<div
					class="flex flex-1 items-center justify-center rounded-box border border-dashed border-base-content/10 bg-base-200/40 p-8 text-center text-sm text-base-content/60"
				>
					Pick a table on the left to inspect its records.
				</div>
			{:else}
				<div class="flex items-center justify-between gap-2">
					<div>
						<h2 class="font-mono text-lg font-semibold">{$recordsStore.table}</h2>
						<p class="text-xs text-base-content/60">
							Showing {$recordsStore.offset + 1}–{Math.min(
								$recordsStore.offset + $recordsStore.records.length,
								$recordsStore.total
							)} of {$recordsStore.total}
						</p>
					</div>
					<div class="flex items-center gap-1">
						<button
							class="btn btn-outline btn-xs"
							onclick={prevPage}
							disabled={$recordsStore.loading || $recordsStore.offset === 0}
						>
							Prev
						</button>
						<button
							class="btn btn-outline btn-xs"
							onclick={nextPage}
							disabled={$recordsStore.loading ||
								$recordsStore.offset + $recordsStore.limit >= $recordsStore.total}
						>
							Next
						</button>
						<button
							class="btn btn-outline btn-xs"
							onclick={refreshCurrent}
							disabled={$recordsStore.loading}
						>
							{$recordsStore.loading ? 'Loading…' : 'Refresh'}
						</button>
					</div>
				</div>

				{#if $recordsStore.error}
					<div class="alert alert-error">
						<span>{$recordsStore.error}</span>
					</div>
				{/if}

				{#if $recordsStore.loading && $recordsStore.records.length === 0}
					<p class="text-sm text-base-content/60">Loading…</p>
				{:else if $recordsStore.records.length === 0}
					<p class="text-sm text-base-content/60">No records in this table.</p>
				{:else}
					<div class="flex min-h-0 flex-1 flex-col gap-2 overflow-y-auto pr-2">
						{#each $recordsStore.records as record, i (recordKey(record, i))}
							<pre
								class="rounded-box border border-base-content/10 bg-base-300 p-3 font-mono text-xs break-all whitespace-pre-wrap">{formatRecord(
									record
								)}</pre>
						{/each}
					</div>
				{/if}
			{/if}
		</section>
	</div>
</div>
