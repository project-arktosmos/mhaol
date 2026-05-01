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
		<div class={collapsible ? 'mt-2' : 'mt-2'}>
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
				<div class="flex flex-col gap-1">
					{#each search.matches as torrent (torrent.infoHash)}
						{@const added = !!torrent.magnetLink && existingHashes.has(torrent.magnetLink)}
						{@const adding = addingHash === torrent.magnetLink}
						{@const ev = rowEval(torrent.magnetLink)}
						<button
							type="button"
							class={classNames(
								'flex flex-wrap items-center gap-2 rounded border border-base-content/10 px-2 py-1 text-left text-xs hover:bg-base-100',
								{ 'opacity-60': added || adding }
							)}
							onclick={() => onAssign(torrent)}
							disabled={addingHash !== null || added}
							title={ev.kind === 'streamable'
								? `Streamable — ${ev.fileName} · ${torrent.title}`
								: ev.kind === 'not-streamable'
									? `Not streamable: ${ev.reason} · ${torrent.title}`
									: torrent.title}
						>
							{#if ev.kind === 'pending' || ev.kind === 'evaluating'}
								<span
									class="loading loading-xs shrink-0 loading-spinner text-base-content/50"
									aria-label="Probing torrent metadata"
								></span>
							{:else if ev.kind === 'streamable'}
								<span
									class="shrink-0 text-success"
									aria-label="Streamable"
									title={`Streamable — ${ev.fileName}`}
								>
									<svg
										xmlns="http://www.w3.org/2000/svg"
										viewBox="0 0 24 24"
										fill="currentColor"
										class="h-3.5 w-3.5"
										aria-hidden="true"
									>
										<polygon points="6 4 20 12 6 20 6 4" />
									</svg>
								</span>
							{:else}
								<span
									class="shrink-0 text-base-content/30"
									aria-label="Not streamable"
									title={`Not streamable: ${ev.reason}`}
								>
									<svg
										xmlns="http://www.w3.org/2000/svg"
										viewBox="0 0 24 24"
										fill="none"
										stroke="currentColor"
										stroke-width="2.5"
										stroke-linecap="round"
										class="h-3.5 w-3.5"
										aria-hidden="true"
									>
										<line x1="5" y1="5" x2="19" y2="19" />
										<line x1="19" y1="5" x2="5" y2="19" />
									</svg>
								</span>
							{/if}
							<span class="font-medium">{torrent.quality ?? '—'}</span>
							<span class="text-success">↑{torrent.seeders}</span>
							<span class="text-warning">↓{torrent.leechers}</span>
							<span class="text-base-content/60">{formatSizeBytes(torrent.sizeBytes)}</span>
							<span class="truncate text-base-content/70"
								>{torrent.parsedTitle || torrent.title}</span
							>
							{#if added}
								<span class="ml-auto">✓</span>
							{:else if adding}
								<span class="ml-auto">…</span>
							{/if}
						</button>
					{/each}
				</div>
			{/if}
		</div>
	{/if}
</div>
