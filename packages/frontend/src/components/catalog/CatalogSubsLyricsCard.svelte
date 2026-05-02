<script lang="ts">
	import classNames from 'classnames';
	import type { SubsLyricsResolver } from '$services/catalog/subs-lyrics-resolver.svelte';
	import type { SubsLyricsItem } from '$types/subs-lyrics.type';

	interface Props {
		resolver: SubsLyricsResolver;
		kind: 'subs' | 'lyrics';
		searchTerm?: string | null;
		onRefresh?: () => void;
	}

	let { resolver, kind, searchTerm, onRefresh }: Props = $props();

	let selected = $state<SubsLyricsItem | null>(null);

	const labelKind = $derived(kind === 'lyrics' ? 'Lyrics' : 'Subtitles');

	const heading = $derived(
		`${labelKind} search${resolver.results.length > 0 ? ` (${resolver.results.length})` : ''}`
	);

	function describe(item: SubsLyricsItem): string {
		if (item.kind === 'lyrics') {
			const parts: string[] = [];
			if ((item.syncedLyrics?.length ?? 0) > 0) parts.push('synced');
			else if (item.plainLyrics) parts.push('plain');
			else if (item.instrumental) parts.push('instrumental');
			if (item.albumName) parts.push(item.albumName);
			return parts.join(' · ');
		}
		const parts: string[] = [];
		if (item.format) parts.push(item.format);
		if (item.isHearingImpaired) parts.push('HI');
		return parts.join(' · ') || item.source;
	}

	function rowLabel(item: SubsLyricsItem): string {
		if (item.kind === 'lyrics') {
			const artist = item.artistName ?? '';
			const track = item.trackName ?? '';
			return [artist, track].filter(Boolean).join(' — ') || item.source;
		}
		// In the subs view rows are nested inside a per-language <details>,
		// so the row label is the OpenSubtitles file id (or source-prefixed
		// fallback) rather than the language name.
		return item.externalId
			? `#${item.externalId}`
			: (item.display ?? item.language ?? item.source);
	}

	function rowKey(item: SubsLyricsItem, index: number): string {
		return `${item.source}::${item.externalId || index}`;
	}

	function languageLabel(item: SubsLyricsItem): string {
		return item.display ?? item.language ?? '—';
	}

	type LanguageGroup = { label: string; items: SubsLyricsItem[] };

	// Group subtitle results by language (display name preferred, falls
	// back to the 3-letter code, then "—"). Ordering: English first if
	// present, then by descending count, then alphabetical.
	const grouped = $derived.by<LanguageGroup[]>(() => {
		if (kind !== 'subs') return [];
		const map = new Map<string, SubsLyricsItem[]>();
		for (const item of resolver.results) {
			const key = languageLabel(item);
			const arr = map.get(key);
			if (arr) arr.push(item);
			else map.set(key, [item]);
		}
		const groups = Array.from(map, ([label, items]) => ({ label, items }));
		groups.sort((a, b) => {
			if (a.label === 'English' && b.label !== 'English') return -1;
			if (b.label === 'English' && a.label !== 'English') return 1;
			if (b.items.length !== a.items.length) return b.items.length - a.items.length;
			return a.label.localeCompare(b.label);
		});
		return groups;
	});
</script>

<div class="card border border-base-content/10 bg-base-200 p-4">
	<div class="flex items-center justify-between gap-2">
		<h2 class="text-sm font-semibold text-base-content/70 uppercase">{heading}</h2>
		{#if onRefresh}
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

	{#if searchTerm}
		<div class="mt-1 flex flex-wrap items-baseline gap-1 text-xs text-base-content/60">
			<span>Query:</span>
			<code class="rounded bg-base-100 px-1 py-0.5 font-mono text-[11px] break-all">
				{searchTerm}
			</code>
		</div>
	{/if}

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
			<div class="flex flex-col gap-1">
				{#each grouped as group, gi (group.label + gi)}
					<details
						class="rounded border border-base-content/10 bg-base-100 open:bg-base-200"
						open={group.label === 'English' || gi === 0}
					>
						<summary
							class="flex cursor-pointer items-center justify-between gap-2 px-2 py-1 text-xs font-medium select-none"
						>
							<span class="truncate">{group.label}</span>
							<span class="text-[10px] text-base-content/60">{group.items.length}</span>
						</summary>
						<ul class="flex flex-col gap-1 border-t border-base-content/10 p-1">
							{#each group.items as item, i (rowKey(item, i))}
								{@const isSelected = selected === item}
								<li>
									<button
										type="button"
										class={classNames(
											'flex w-full flex-col gap-0.5 rounded border border-base-content/10 bg-base-100 px-2 py-1 text-left text-xs hover:bg-base-200',
											{ 'border-primary': isSelected }
										)}
										onclick={() => (selected = isSelected ? null : item)}
									>
										<span class="truncate font-medium">{rowLabel(item)}</span>
										<span class="truncate text-[10px] text-base-content/60">
											{describe(item)}
										</span>
									</button>
								</li>
							{/each}
						</ul>
					</details>
				{/each}
			</div>
		{:else}
			<ul class="flex flex-col gap-1">
				{#each resolver.results as item, i (rowKey(item, i))}
					{@const isSelected = selected === item}
					<li>
						<button
							type="button"
							class={classNames(
								'flex w-full flex-col gap-0.5 rounded border border-base-content/10 bg-base-100 px-2 py-1 text-left text-xs hover:bg-base-200',
								{ 'border-primary': isSelected }
							)}
							onclick={() => (selected = isSelected ? null : item)}
						>
							<span class="truncate font-medium">{rowLabel(item)}</span>
							<span class="truncate text-[10px] text-base-content/60">{describe(item)}</span>
						</button>
					</li>
				{/each}
			</ul>

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
		{/if}
	</div>
</div>
