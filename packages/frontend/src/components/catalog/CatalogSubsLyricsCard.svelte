<script lang="ts">
	import classNames from 'classnames';
	import type { SubsLyricsResolver } from '$services/catalog/subs-lyrics-resolver.svelte';
	import type { SubsLyricsItem } from '$types/subs-lyrics.type';

	interface Props {
		resolver: SubsLyricsResolver;
		kind: 'subs' | 'lyrics';
		onRefresh?: () => void;
	}

	let { resolver, kind, onRefresh }: Props = $props();

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
		return item.display ?? item.language ?? item.source;
	}

	function rowKey(item: SubsLyricsItem, index: number): string {
		return `${item.source}::${item.externalId || index}`;
	}
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
