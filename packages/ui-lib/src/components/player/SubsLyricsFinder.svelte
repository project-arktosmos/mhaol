<script lang="ts">
	import classNames from 'classnames';
	import { playerService } from 'ui-lib/services/player.service';
	import { subsLyricsFinderService } from 'ui-lib/services/subs-lyrics-finder.service';
	import type { SubsLyricsItem, SubsLyricsSearchType } from 'ui-lib/types/subs-lyrics.type';

	const playerState = playerService.state;
	const finderState = subsLyricsFinderService.state;

	const SEARCH_TYPES: { id: SubsLyricsSearchType; label: string }[] = [
		{ id: 'track', label: 'Track' },
		{ id: 'album', label: 'Album' },
		{ id: 'movie', label: 'Movie' },
		{ id: 'tv show', label: 'TV show' },
		{ id: 'tv episode', label: 'TV episode' }
	];

	let lastFileId: string | null = $state(null);

	$effect(() => {
		const file = $playerState.currentFile;
		if (!file) {
			if (lastFileId !== null) {
				lastFileId = null;
				subsLyricsFinderService.clear();
			}
			return;
		}
		if (file.id === lastFileId) return;
		lastFileId = file.id;
		const guessedType: SubsLyricsSearchType = file.mode === 'audio' ? 'track' : 'movie';
		subsLyricsFinderService.setType(guessedType);
		subsLyricsFinderService.setQuery(file.name ?? '');
		subsLyricsFinderService.setExternalId('');
	});

	function setType(value: SubsLyricsSearchType) {
		subsLyricsFinderService.setType(value);
	}

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

	function label(item: SubsLyricsItem): string {
		if (item.kind === 'lyrics') {
			const artist = item.artistName ?? '';
			const track = item.trackName ?? '';
			return [artist, track].filter(Boolean).join(' — ') || item.source;
		}
		return item.display ?? item.language ?? item.source;
	}
</script>

<section class="flex flex-col gap-2 rounded-box border border-base-content/10 bg-base-300 p-3">
	<header class="flex items-center justify-between gap-2">
		<h3 class="text-sm font-semibold">Subs / Lyrics</h3>
		{#if $playerState.currentFile}
			<span class="truncate text-xs text-base-content/60" title={$playerState.currentFile.name}>
				{$playerState.currentFile.name}
			</span>
		{/if}
	</header>

	{#if !$playerState.currentFile}
		<p class="text-xs text-base-content/60">Play something to search subs or lyrics.</p>
	{:else}
		<div class="flex flex-wrap gap-1">
			{#each SEARCH_TYPES as option (option.id)}
				<button
					type="button"
					class={classNames('btn btn-xs', {
						'btn-primary': $finderState.type === option.id,
						'btn-ghost': $finderState.type !== option.id
					})}
					onclick={() => setType(option.id)}
				>
					{option.label}
				</button>
			{/each}
		</div>

		<input
			type="text"
			class="input-bordered input input-sm w-full"
			placeholder="Title or query"
			value={$finderState.query}
			oninput={(e) => subsLyricsFinderService.setQuery(e.currentTarget.value)}
		/>

		{#if $finderState.type === 'movie' || $finderState.type === 'tv show' || $finderState.type === 'tv season' || $finderState.type === 'tv episode'}
			<input
				type="text"
				class="input-bordered input input-sm w-full"
				placeholder="TMDB id"
				value={$finderState.externalId}
				oninput={(e) => subsLyricsFinderService.setExternalId(e.currentTarget.value)}
			/>
		{/if}

		<button
			type="button"
			class={classNames('btn btn-sm btn-primary', {
				'btn-disabled': $finderState.searching
			})}
			disabled={$finderState.searching}
			onclick={() => subsLyricsFinderService.search()}
		>
			{$finderState.searching ? 'Searching…' : 'Search'}
		</button>

		{#if $finderState.error}
			<p class="text-xs text-error">{$finderState.error}</p>
		{/if}

		{#if $finderState.results.length > 0}
			<ul class="flex max-h-64 flex-col gap-1 overflow-y-auto">
				{#each $finderState.results as item (item.source + '::' + item.externalId)}
					{@const isSelected = $finderState.selected === item}
					<li>
						<button
							type="button"
							class={classNames(
								'flex w-full flex-col gap-0.5 rounded border border-base-content/10 bg-base-100 px-2 py-1 text-left text-xs hover:bg-base-200',
								{ 'border-primary': isSelected }
							)}
							onclick={() => subsLyricsFinderService.select(isSelected ? null : item)}
						>
							<span class="truncate font-medium">{label(item)}</span>
							<span class="truncate text-[10px] text-base-content/60">{describe(item)}</span>
						</button>
					</li>
				{/each}
			</ul>
		{/if}

		{#if $finderState.selected}
			{@const sel = $finderState.selected}
			<div class="flex flex-col gap-1 rounded border border-base-content/10 bg-base-100 p-2">
				{#if sel.kind === 'lyrics'}
					{#if sel.syncedLyrics && sel.syncedLyrics.length > 0}
						<div class="flex max-h-64 flex-col gap-0.5 overflow-y-auto text-xs leading-tight">
							{#each sel.syncedLyrics as line (line.time)}
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
</section>
