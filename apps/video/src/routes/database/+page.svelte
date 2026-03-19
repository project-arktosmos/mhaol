<script lang="ts">
	import { onMount } from 'svelte';
	import classNames from 'classnames';
	import { apiUrl } from '$lib/api-base';

	interface TableInfo {
		name: string;
		columns: { name: string; type: string }[];
		rowCount: number;
	}

	interface TableData {
		table: string;
		columns: { name: string; type: string }[];
		rows: Record<string, unknown>[];
		pagination: {
			page: number;
			limit: number;
			total: number;
			totalPages: number;
		};
	}

	let tables = $state<TableInfo[]>([]);
	let activeTable = $state<string | null>(null);
	let tableData = $state<TableData | null>(null);
	let loading = $state(false);
	let tableLoading = $state(false);
	let error = $state<string | null>(null);

	onMount(() => {
		loadTables();
	});

	async function loadTables() {
		loading = true;
		error = null;

		try {
			const res = await fetch(apiUrl('/api/database/tables'));
			if (!res.ok) throw new Error('Failed to load tables');
			tables = await res.json();

			if (tables.length > 0 && !activeTable) {
				selectTable(tables[0].name);
			}
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	}

	async function selectTable(name: string, page: number = 1) {
		activeTable = name;
		tableLoading = true;
		error = null;

		try {
			const res = await fetch(
				apiUrl(`/api/database/tables/${encodeURIComponent(name)}?page=${page}&limit=20`)
			);
			if (!res.ok) throw new Error(`Failed to load table "${name}"`);
			tableData = await res.json();
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			tableLoading = false;
		}
	}

	function handlePageChange(page: number) {
		if (activeTable) {
			selectTable(activeTable, page);
		}
	}

	function formatCell(value: unknown): string {
		if (value === null || value === undefined) return 'NULL';
		if (typeof value === 'object') return JSON.stringify(value);
		return String(value);
	}

	function truncateCell(value: string, maxLength: number = 80): string {
		if (value.length <= maxLength) return value;
		return value.slice(0, maxLength) + '...';
	}
</script>

<div class="container mx-auto p-4">
	<div class="mb-6">
		<h1 class="text-3xl font-bold">Database</h1>
		<p class="text-sm opacity-70">Browse SQLite tables and their contents</p>
	</div>

	{#if error}
		<div class="mb-4 alert alert-error">
			<span>{error}</span>
			<button class="btn btn-ghost btn-sm" onclick={() => (error = null)}>x</button>
		</div>
	{/if}

	{#if loading}
		<div class="flex justify-center py-12">
			<span class="loading loading-lg loading-spinner"></span>
		</div>
	{:else if tables.length === 0}
		<div class="rounded-lg bg-base-200 p-8 text-center">
			<p class="opacity-50">No tables found in the database.</p>
		</div>
	{:else}
		<div class="flex gap-4">
			<ul class="menu w-56 shrink-0 rounded-lg bg-base-200">
				{#each tables as table (table.name)}
					<li>
						<button
							class={classNames('justify-between', {
								active: activeTable === table.name
							})}
							onclick={() => selectTable(table.name)}
						>
							{table.name}
							<span class="badge badge-sm">{table.rowCount}</span>
						</button>
					</li>
				{/each}
			</ul>

			<div class="min-w-0 flex-1">
				{#if tableLoading}
					<div class="flex justify-center py-12">
						<span class="loading loading-lg loading-spinner"></span>
					</div>
				{:else if tableData}
					<div class="mb-2 flex items-center justify-between">
						<h2 class="text-lg font-semibold">{tableData.table}</h2>
						<span class="text-sm opacity-70">
							{tableData.pagination.total} row{tableData.pagination.total !== 1 ? 's' : ''}
						</span>
					</div>

					{#if tableData.rows.length === 0}
						<div class="rounded-lg bg-base-200 p-8 text-center">
							<p class="opacity-50">This table is empty.</p>
						</div>
					{:else}
						<div class="overflow-x-auto rounded-lg border border-base-300">
							<table class="table table-sm">
								<thead>
									<tr class="bg-base-200">
										{#each tableData.columns as col (col.name)}
											<th>
												<div>{col.name}</div>
												<div class="text-xs font-normal opacity-50">{col.type}</div>
											</th>
										{/each}
									</tr>
								</thead>
								<tbody>
									{#each tableData.rows as row, i (i)}
										<tr class="hover:bg-base-200/50">
											{#each tableData.columns as col (col.name)}
												<td
													class={classNames('max-w-xs', {
														'italic opacity-40':
															row[col.name] === null || row[col.name] === undefined
													})}
													title={formatCell(row[col.name])}
												>
													{truncateCell(formatCell(row[col.name]))}
												</td>
											{/each}
										</tr>
									{/each}
								</tbody>
							</table>
						</div>

						{#if tableData.pagination.totalPages > 1}
							<div class="mt-4 flex justify-center gap-2">
								<button
									class="btn btn-sm"
									disabled={tableData.pagination.page <= 1}
									onclick={() => handlePageChange(tableData!.pagination.page - 1)}
								>
									Previous
								</button>
								<span class="flex items-center px-4 text-sm">
									Page {tableData.pagination.page} of {tableData.pagination.totalPages}
								</span>
								<button
									class="btn btn-sm"
									disabled={tableData.pagination.page >= tableData.pagination.totalPages}
									onclick={() => handlePageChange(tableData!.pagination.page + 1)}
								>
									Next
								</button>
							</div>
						{/if}
					{/if}
				{/if}
			</div>
		</div>
	{/if}
</div>
