<script lang="ts">
	import classNames from 'classnames';
	import { formatSizeBytes, type TorrentResultItem } from '$lib/search.service';
	import type { TorrentSearch, TorrentRowEval } from '$services/catalog/torrent-search.svelte';

	interface Props {
		search: TorrentSearch;
		onAssign: (torrent: TorrentResultItem) => void | Promise<void>;
		addingHash: string | null;
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
		addingHash,
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
								<th class="w-56">Streamability</th>
								<th class="w-20">Quality</th>
								<th class="w-20 text-success">Seeders</th>
								<th class="w-20 text-warning">Leechers</th>
								<th class="w-20">Size</th>
								<th>Title</th>
								<th class="w-16"></th>
							</tr>
						</thead>
						<tbody>
							{#each search.groupedMatches as group (group.label)}
								<tr class="bg-base-300/40">
									<th
										colspan="7"
										class="text-xs font-semibold tracking-wider text-base-content/70 uppercase"
									>
										{group.label} ({group.rows.length})
									</th>
								</tr>
								{#each group.rows as torrent (torrent.infoHash)}
									{@const added = !!torrent.magnetLink && existingHashes.has(torrent.magnetLink)}
									{@const adding = addingHash === torrent.magnetLink}
									{@const ev = rowEval(torrent.magnetLink)}
									<tr class={classNames('hover', { 'opacity-60': added || adding })}>
										<td>
											{#if ev.kind === 'pending'}
												<span class="text-xs text-base-content/50">Queued…</span>
											{:else if ev.kind === 'evaluating'}
												<div class="flex items-center gap-2">
													<span
														class="loading loading-xs shrink-0 loading-spinner text-base-content/50"
														aria-hidden="true"
													></span>
													<span class="text-xs text-base-content/60">Probing…</span>
												</div>
											{:else if ev.kind === 'streamable'}
												<div class="flex flex-col gap-0.5">
													<span class="badge gap-1 badge-sm badge-success">
														<svg
															xmlns="http://www.w3.org/2000/svg"
															viewBox="0 0 24 24"
															fill="currentColor"
															class="h-3 w-3"
															aria-hidden="true"
														>
															<polygon points="6 4 20 12 6 20 6 4" />
														</svg>
														Streamable
													</span>
													<span
														class="max-w-[14rem] truncate text-[10px] text-base-content/60"
														title={ev.fileName}
													>
														{ev.fileName}
													</span>
												</div>
											{:else if ev.kind === 'skipped'}
												<span
													class="text-xs text-base-content/40"
													title="Skipped — a streamable candidate was found earlier in this quality group"
												>
													—
												</span>
											{:else}
												<div class="flex flex-col gap-0.5">
													<span class="badge gap-1 badge-ghost badge-sm">
														<svg
															xmlns="http://www.w3.org/2000/svg"
															viewBox="0 0 24 24"
															fill="none"
															stroke="currentColor"
															stroke-width="2.5"
															stroke-linecap="round"
															class="h-3 w-3 text-base-content/40"
															aria-hidden="true"
														>
															<line x1="5" y1="5" x2="19" y2="19" />
															<line x1="19" y1="5" x2="5" y2="19" />
														</svg>
														Not streamable
													</span>
													<span
														class="max-w-[14rem] truncate text-[10px] text-base-content/60"
														title={ev.reason}
													>
														{ev.reason}
													</span>
												</div>
											{/if}
										</td>
										<td class="text-xs font-medium">{torrent.quality ?? '—'}</td>
										<td class="text-xs text-success">{torrent.seeders}</td>
										<td class="text-xs text-warning">{torrent.leechers}</td>
										<td class="text-xs text-base-content/70"
											>{formatSizeBytes(torrent.sizeBytes)}</td
										>
										<td
											class="max-w-md truncate text-xs text-base-content/80"
											title={torrent.title}
										>
											{torrent.parsedTitle || torrent.title}
										</td>
										<td class="text-right">
											{#if added}
												<span class="badge badge-sm badge-success">added</span>
											{:else}
												<button
													type="button"
													class="btn btn-xs btn-primary"
													disabled={addingHash !== null}
													onclick={() => onAssign(torrent)}
													aria-label="Use this torrent"
												>
													{adding ? '…' : 'Use'}
												</button>
											{/if}
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
