<script lang="ts">
	import classNames from 'classnames';
	import { formatSizeBytes, type TorrentResultItem } from '$lib/search.service';
	import type { TorrentSearch, TorrentRowEval } from '$services/catalog/torrent-search.svelte';

	interface Props {
		search: TorrentSearch;
		onAssign: (torrent: TorrentResultItem) => void | Promise<void>;
		onStream?: (torrent: TorrentResultItem) => void | Promise<void>;
		addingHash: string | null;
		streamingHash?: string | null;
		assignError?: string | null;
		existingHashes?: Set<string>;
		collapsible?: boolean;
		open?: boolean;
		onToggle?: () => void;
		onRefresh?: () => void;
	}

	let {
		search,
		onAssign,
		onStream,
		addingHash,
		streamingHash = null,
		assignError = null,
		existingHashes = new Set<string>(),
		collapsible = false,
		open = true,
		onToggle,
		onRefresh
	}: Props = $props();

	function rowEval(magnet: string | undefined | null): TorrentRowEval {
		if (!magnet) return { kind: 'not-streamable', reason: 'no magnet' };
		return search.rowEvals[magnet] ?? { kind: 'pending' };
	}

	function firstStreamableIndex(rows: TorrentResultItem[]): number {
		for (let i = 0; i < rows.length; i++) {
			if (rowEval(rows[i].magnetLink).kind === 'streamable') return i;
		}
		return -1;
	}

	function hasInflight(rows: TorrentResultItem[]): boolean {
		for (const r of rows) {
			const k = rowEval(r.magnetLink).kind;
			if (k === 'pending' || k === 'evaluating') return true;
		}
		return false;
	}

	function activeProbeIndex(rows: TorrentResultItem[]): number {
		for (let i = 0; i < rows.length; i++) {
			if (rowEval(rows[i].magnetLink).kind === 'evaluating') return i;
		}
		for (let i = 0; i < rows.length; i++) {
			if (rowEval(rows[i].magnetLink).kind === 'pending') return i;
		}
		return -1;
	}

	let expandedGroups = $state<Record<string, boolean>>({});

	function toggleGroup(label: string) {
		const wasExpanded = expandedGroups[label] ?? false;
		expandedGroups = { ...expandedGroups, [label]: !wasExpanded };
		if (!wasExpanded) void search.probeRemaining(label);
	}

	const heading = $derived(
		`Torrent search${(!collapsible || open) && search.matches.length > 0 ? ` (${search.matches.length})` : ''}`
	);
</script>

<div class="card border border-base-content/10 bg-base-200 p-4">
	<div class="flex items-center justify-between gap-2">
		{#if collapsible}
			<button
				type="button"
				class="flex flex-1 items-center gap-2 text-left"
				onclick={() => onToggle?.()}
				aria-expanded={open}
			>
				<span class="text-base-content/60" aria-hidden="true">{open ? '▼' : '▶'}</span>
				<h2 class="text-sm font-semibold text-base-content/70 uppercase">{heading}</h2>
			</button>
		{:else}
			<h2 class="text-sm font-semibold text-base-content/70 uppercase">{heading}</h2>
		{/if}
		{#if (!collapsible || open) && onRefresh}
			<button
				type="button"
				class="btn btn-outline btn-xs"
				onclick={() => onRefresh?.()}
				disabled={search.status === 'searching'}
			>
				{search.status === 'searching' ? 'Searching…' : 'Refresh'}
			</button>
		{/if}
	</div>
	{#if !collapsible || open}
		<div class="mt-2">
			{#if assignError}
				<div class="mb-2 alert alert-error">
					<span>{assignError}</span>
				</div>
			{/if}
			{#if search.status === 'searching' && search.matches.length === 0}
				<p class="text-sm text-base-content/60">Searching…</p>
			{:else if search.status === 'error'}
				<p class="text-sm text-error">{search.error ?? 'Failed'}</p>
			{:else if search.matches.length === 0}
				<p class="text-sm text-base-content/60">No matching torrents.</p>
			{:else}
				<div class="overflow-x-auto rounded-box border border-base-content/10">
					<table class="table table-xs">
						<thead>
							<tr>
								<th>Title</th>
								<th class="w-12 text-success" title="Seeders">↑</th>
								<th class="w-12 text-warning" title="Leechers">↓</th>
								<th class="w-20">Size</th>
								<th class="w-32"></th>
							</tr>
						</thead>
						<tbody>
							{#each search.groupedMatches as group (group.label)}
								{@const streamableIdx = firstStreamableIndex(group.rows)}
								{@const probing = group.probe && hasInflight(group.rows)}
								{@const probeIdx = probing ? activeProbeIndex(group.rows) : -1}
								{@const defaultCollapsed = !group.probe}
								{@const expanded = expandedGroups[group.label] ?? false}
								<tr class="bg-base-300/40">
									<th colspan="5" class="p-0">
										<button
											type="button"
											class="flex w-full items-center gap-2 px-3 py-2 text-left text-xs font-semibold tracking-wider text-base-content/70 uppercase hover:bg-base-300/60"
											onclick={() => toggleGroup(group.label)}
											aria-expanded={expanded}
										>
											<span aria-hidden="true">{expanded ? '▼' : '▶'}</span>
											<span>{group.label} ({group.rows.length})</span>
										</button>
									</th>
								</tr>
								{#each group.rows as torrent, rowIdx (torrent.infoHash)}
									{@const added = !!torrent.magnetLink && existingHashes.has(torrent.magnetLink)}
									{@const adding = addingHash === torrent.magnetLink}
									{@const streaming = streamingHash === torrent.magnetLink}
									{@const ev = rowEval(torrent.magnetLink)}
									{@const hidden = expanded
										? false
										: defaultCollapsed
											? true
											: probing
												? rowIdx !== probeIdx
												: streamableIdx >= 0 && rowIdx > streamableIdx}
									<tr
										class={classNames('hover', {
											'opacity-60': added || adding,
											hidden
										})}
									>
										<td>
											<div class="flex flex-col gap-0.5">
												<span
													class="block max-w-[18rem] truncate text-xs text-base-content/80"
													title={torrent.title}
												>
													{torrent.parsedTitle || torrent.title}
												</span>
												{#if ev.kind === 'pending'}
													<span class="text-[10px] text-base-content/50">Queued…</span>
												{:else if ev.kind === 'evaluating'}
													<div class="flex items-center gap-1.5">
														<span
															class="loading loading-xs shrink-0 loading-spinner text-base-content/50"
															aria-hidden="true"
														></span>
														<span class="text-[10px] text-base-content/60">Probing…</span>
													</div>
												{:else if ev.kind === 'streamable'}
													<span
														class="block max-w-[18rem] truncate text-[10px] text-success"
														title={ev.fileName}
													>
														{ev.fileName}
													</span>
												{:else if ev.kind === 'skipped'}
													<span class="text-[10px] text-base-content/40" title={ev.reason}>—</span>
												{:else}
													<span
														class="block max-w-[18rem] truncate text-[10px] text-base-content/50"
														title={ev.reason}
													>
														Not streamable: {ev.reason}
													</span>
												{/if}
											</div>
										</td>
										<td class="text-xs text-success">{torrent.seeders}</td>
										<td class="text-xs text-warning">{torrent.leechers}</td>
										<td class="text-xs text-base-content/70"
											>{formatSizeBytes(torrent.sizeBytes)}</td
										>
										<td class="text-right">
											<div class="flex items-center justify-end gap-1">
												{#if ev.kind === 'streamable' && onStream}
													<button
														type="button"
														class="btn btn-xs btn-success"
														disabled={streamingHash !== null}
														onclick={() => onStream?.(torrent)}
														aria-label="Stream this torrent"
													>
														{streaming ? '…' : 'Stream'}
													</button>
												{/if}
												{#if added}
													<span class="badge badge-sm badge-success">added</span>
												{:else}
													<button
														type="button"
														class="btn btn-xs btn-primary"
														disabled={addingHash !== null}
														onclick={() => onAssign(torrent)}
														aria-label="Download this torrent"
													>
														{adding ? '…' : 'Download'}
													</button>
												{/if}
											</div>
										</td>
									</tr>
								{/each}
							{/each}
						</tbody>
					</table>
				</div>
			{/if}
		</div>
	{/if}
</div>
