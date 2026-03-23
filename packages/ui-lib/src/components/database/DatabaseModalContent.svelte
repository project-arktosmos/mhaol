<script lang="ts">
	import { onMount } from 'svelte';
	import classNames from 'classnames';
	import { databaseService } from 'ui-lib/services/database.service';

	const dbState = databaseService.state;

	let selectedTableName = $state<string | null>(null);

	onMount(() => {
		databaseService.fetchTables();
	});

	async function selectTable(name: string) {
		selectedTableName = name;
		await databaseService.fetchTableRows(name);
	}

	async function goToPage(page: number) {
		if (!selectedTableName) return;
		await databaseService.fetchTableRows(
			selectedTableName,
			page,
			$dbState.selectedTable?.pagination.limit
		);
	}

	function goBack() {
		selectedTableName = null;
		databaseService.clearSelection();
	}

	function truncateValue(val: unknown): string {
		if (val === null || val === undefined) return 'NULL';
		const str = String(val);
		return str.length > 120 ? str.slice(0, 120) + '...' : str;
	}
</script>

<div class="pr-8">
	<h3 class="text-lg font-bold">Database</h3>
	<p class="text-sm text-base-content/60">Browse database tables and rows</p>
</div>

{#if $dbState.error}
	<div class="mt-4 alert alert-error">
		<span>{$dbState.error}</span>
	</div>
{/if}

{#if !selectedTableName}
	<!-- Table list -->
	<div class="mt-4 overflow-x-auto">
		<table class="table w-full table-zebra table-sm">
			<thead>
				<tr>
					<th>Table</th>
					<th class="text-right">Rows</th>
					<th class="text-right">Columns</th>
				</tr>
			</thead>
			<tbody>
				{#each $dbState.tables as table}
					<tr class="cursor-pointer hover:bg-base-200" onclick={() => selectTable(table.name)}>
						<td class="font-mono text-sm">{table.name}</td>
						<td class="text-right">{table.rowCount}</td>
						<td class="text-right">{table.columns.length}</td>
					</tr>
				{/each}
			</tbody>
		</table>
	</div>

	{#if $dbState.loading}
		<div class="mt-4 flex justify-center">
			<span class="loading loading-md loading-spinner"></span>
		</div>
	{/if}
{:else if $dbState.selectedTable}
	<!-- Table detail view -->
	<div class="mt-4">
		<div class="flex items-center gap-2">
			<button class="btn btn-ghost btn-sm" onclick={goBack}>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					fill="none"
					viewBox="0 0 24 24"
					stroke-width="1.5"
					stroke="currentColor"
					class="h-4 w-4"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						d="M10.5 19.5 3 12m0 0 7.5-7.5M3 12h18"
					/>
				</svg>
			</button>
			<h4 class="font-mono text-base font-semibold">{$dbState.selectedTable.table}</h4>
			<span class="badge badge-ghost badge-sm">
				{$dbState.selectedTable.pagination.total} rows
			</span>
		</div>

		<div class="mt-3 overflow-x-auto rounded-lg border border-base-300">
			<table class="table w-full table-xs">
				<thead>
					<tr>
						{#each $dbState.selectedTable.columns as col}
							<th class="whitespace-nowrap">
								<span>{col.name}</span>
								<span class="ml-1 font-normal text-base-content/40">{col.type}</span>
							</th>
						{/each}
					</tr>
				</thead>
				<tbody>
					{#each $dbState.selectedTable.rows as row}
						<tr class="hover:bg-base-200">
							{#each $dbState.selectedTable.columns as col}
								<td
									class={classNames('max-w-xs truncate font-mono text-xs whitespace-nowrap', {
										'text-base-content/30': row[col.name] === null
									})}
									title={String(row[col.name] ?? 'NULL')}
								>
									{truncateValue(row[col.name])}
								</td>
							{/each}
						</tr>
					{/each}
					{#if $dbState.selectedTable.rows.length === 0}
						<tr>
							<td
								colspan={$dbState.selectedTable.columns.length}
								class="text-center text-base-content/50"
							>
								No rows
							</td>
						</tr>
					{/if}
				</tbody>
			</table>
		</div>

		<!-- Pagination -->
		{#if $dbState.selectedTable.pagination.totalPages > 1}
			<div class="mt-3 flex items-center justify-center gap-2">
				<button
					class="btn btn-ghost btn-sm"
					disabled={$dbState.selectedTable.pagination.page <= 1 || $dbState.loading}
					onclick={() => goToPage($dbState.selectedTable!.pagination.page - 1)}
				>
					Prev
				</button>
				<span class="text-sm">
					Page {$dbState.selectedTable.pagination.page} of {$dbState.selectedTable.pagination
						.totalPages}
				</span>
				<button
					class="btn btn-ghost btn-sm"
					disabled={$dbState.selectedTable.pagination.page >=
						$dbState.selectedTable.pagination.totalPages || $dbState.loading}
					onclick={() => goToPage($dbState.selectedTable!.pagination.page + 1)}
				>
					Next
				</button>
			</div>
		{/if}
	</div>

	{#if $dbState.loading}
		<div class="mt-4 flex justify-center">
			<span class="loading loading-md loading-spinner"></span>
		</div>
	{/if}
{/if}
