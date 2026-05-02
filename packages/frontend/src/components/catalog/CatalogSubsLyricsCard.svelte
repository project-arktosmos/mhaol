<script lang="ts">
	import classNames from 'classnames';
	import type { SubsLyricsResolver } from '$services/catalog/subs-lyrics-resolver.svelte';
	import type { SubsLyricsItem } from '$types/subs-lyrics.type';

	interface Props {
		resolver: SubsLyricsResolver;
		kind: 'subs' | 'lyrics';
		searchTerm?: string | null;
		onRefresh?: () => void;
		collapsible?: boolean;
		open?: boolean;
		onToggle?: () => void;
	}

	let {
		resolver,
		kind,
		searchTerm,
		onRefresh,
		collapsible = false,
		open = true,
		onToggle
	}: Props = $props();

	let selected = $state<SubsLyricsItem | null>(null);

	const labelKind = $derived(kind === 'lyrics' ? 'Lyrics' : 'Subtitles');

	const heading = $derived(
		`${labelKind} search${resolver.results.length > 0 ? ` (${resolver.results.length})` : ''}`
	);

	function rowKey(item: SubsLyricsItem, index: number): string {
		return `${item.source}::${item.externalId || index}`;
	}

	function languageLabel(item: SubsLyricsItem): string {
		return item.display ?? item.language ?? '—';
	}

	function filenameExt(item: SubsLyricsItem): string | null {
		const name = item.release;
		if (!name) return null;
		const dot = name.lastIndexOf('.');
		if (dot < 0 || dot === name.length - 1) return null;
		return name.slice(dot + 1);
	}

	function lyricsType(item: SubsLyricsItem): string {
		if ((item.syncedLyrics?.length ?? 0) > 0) return 'synced';
		if (item.plainLyrics) return 'plain';
		if (item.instrumental) return 'instrumental';
		return '—';
	}

	// Sort subtitle results so all rows for the same language are
	// adjacent. The backend filters down to English / Catalan / Spanish;
	// here we just enforce the user's preference order. Within a language
	// the original order is preserved.
	const LANGUAGE_PRIORITY: Record<string, number> = {
		English: 0,
		Catalan: 1,
		Spanish: 2
	};
	const sortedSubs = $derived.by<SubsLyricsItem[]>(() => {
		if (kind !== 'subs') return [];
		return resolver.results
			.map((item, idx) => ({ item, idx, key: languageLabel(item) }))
			.sort((a, b) => {
				const ra = LANGUAGE_PRIORITY[a.key] ?? 99;
				const rb = LANGUAGE_PRIORITY[b.key] ?? 99;
				if (ra !== rb) return ra - rb;
				if (a.key !== b.key) return a.key.localeCompare(b.key);
				return a.idx - b.idx;
			})
			.map((entry) => entry.item);
	});

	function languageOf(item: SubsLyricsItem, index: number): string | null {
		if (kind !== 'subs') return null;
		const label = languageLabel(item);
		if (index === 0) return label;
		const prev = sortedSubs[index - 1];
		return prev && languageLabel(prev) === label ? null : label;
	}
</script>

<div class="flex flex-col gap-2">
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
				disabled={resolver.status === 'searching'}
			>
				{resolver.status === 'searching' ? 'Searching…' : 'Refresh'}
			</button>
		{/if}
	</div>

	{#if (!collapsible || open) && searchTerm}
		<div class="mt-1 flex flex-wrap items-baseline gap-1 text-xs text-base-content/60">
			<span>Query:</span>
			<code class="rounded bg-base-100 px-1 py-0.5 font-mono text-[11px] break-all">
				{searchTerm}
			</code>
		</div>
	{/if}

	{#if !collapsible || open}
		<div class="mt-2">
			{#if resolver.status === 'searching' && resolver.results.length === 0}
				<p class="text-sm text-base-content/60">Searching…</p>
			{:else if resolver.status === 'error'}
				<p class="text-sm text-error">{resolver.error ?? 'Failed'}</p>
			{:else if resolver.status === 'done' && resolver.results.length === 0}
				<p class="text-sm text-base-content/60">
					No {kind === 'lyrics' ? 'lyrics' : 'subtitles'} found.
				</p>
			{:else if resolver.status === 'idle'}
				<p class="text-sm text-base-content/60">Idle.</p>
			{:else if kind === 'subs'}
				<div class="max-h-96 overflow-y-auto rounded border border-base-content/10 bg-base-100">
					<table class="table table-xs">
						<thead class="sticky top-0 bg-base-200 text-[10px] uppercase">
							<tr>
								<th class="w-28">Language</th>
								<th>Filename</th>
								<th class="w-12">Ext</th>
								<th class="w-10">HI</th>
								<th class="w-24">ID</th>
							</tr>
						</thead>
						<tbody>
							{#each sortedSubs as item, i (rowKey(item, i))}
								{@const lang = languageOf(item, i)}
								{@const ext = filenameExt(item)}
								<tr
									class={classNames('cursor-pointer hover:bg-base-200', {
										'bg-primary/10': selected === item
									})}
									onclick={() => (selected = selected === item ? null : item)}
								>
									<td class="font-medium">{lang ?? ''}</td>
									<td
										class="max-w-[1px] truncate text-[11px] text-base-content/70"
										title={item.release ?? ''}
									>
										{item.release ?? '…'}
									</td>
									<td class="font-mono text-[10px] text-base-content/60 uppercase">
										{ext ?? item.format ?? '—'}
									</td>
									<td class="text-center">{item.isHearingImpaired ? 'HI' : ''}</td>
									<td class="font-mono text-[10px] text-base-content/60">{item.externalId}</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			{:else}
				<div class="max-h-96 overflow-y-auto rounded border border-base-content/10 bg-base-100">
					<table class="table table-xs">
						<thead class="sticky top-0 bg-base-200 text-[10px] uppercase">
							<tr>
								<th>Track</th>
								<th>Artist</th>
								<th>Album</th>
								<th class="w-24">Type</th>
								<th class="w-20">Source</th>
							</tr>
						</thead>
						<tbody>
							{#each resolver.results as item, i (rowKey(item, i))}
								<tr
									class={classNames('cursor-pointer hover:bg-base-200', {
										'bg-primary/10': selected === item
									})}
									onclick={() => (selected = selected === item ? null : item)}
								>
									<td class="font-medium">{item.trackName ?? '—'}</td>
									<td class="text-base-content/70">{item.artistName ?? '—'}</td>
									<td class="text-base-content/70">{item.albumName ?? '—'}</td>
									<td>{lyricsType(item)}</td>
									<td class="text-base-content/70">{item.source}</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			{/if}

			{#if selected}
				{@const sel = selected}
				<div class="mt-2 flex flex-col gap-1 rounded border border-base-content/10 bg-base-100 p-2">
					{#if sel.kind === 'lyrics'}
						{#if sel.syncedLyrics && sel.syncedLyrics.length > 0}
							<div class="flex max-h-64 flex-col gap-0.5 overflow-y-auto text-xs leading-tight">
								{#each sel.syncedLyrics as line, idx (idx)}
									<span class="text-base-content/80">{line.text || '…'}</span>
								{/each}
							</div>
						{:else if sel.plainLyrics}
							<pre
								class="max-h-64 overflow-y-auto text-xs whitespace-pre-wrap text-base-content/80">{sel.plainLyrics}</pre>
						{:else if sel.instrumental}
							<span class="text-xs text-base-content/60">Instrumental.</span>
						{:else}
							<span class="text-xs text-base-content/60">No lyrics in this entry.</span>
						{/if}
					{:else if sel.url}
						<a
							class="link text-xs break-all link-primary"
							href={sel.url}
							target="_blank"
							rel="noopener"
						>
							Open subtitle ({sel.format ?? 'file'})
						</a>
					{:else}
						<span class="text-xs text-base-content/60">No URL provided.</span>
					{/if}
				</div>
			{/if}
		</div>
	{/if}
</div>
