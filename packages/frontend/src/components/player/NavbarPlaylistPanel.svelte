<script lang="ts">
	import classNames from 'classnames';
	import { tick } from 'svelte';
	import { Icon } from 'cloud-ui';
	import { playerService } from '$services/player.service';
	import { playPlaylistTrack } from '$lib/youtube-match.service';

	const playerState = playerService.state;
	const playerDisplayMode = playerService.displayMode;
	const playlist = playerService.playlist;

	let listContainer: HTMLDivElement | null = $state(null);
	let switchingIndex: number | null = $state(null);
	let switchError: string | null = $state(null);
	let collapsed = $state(false);

	let visible = $derived(
		$playerDisplayMode === 'navbar' &&
			$playerState.currentFile !== null &&
			$playlist !== null &&
			$playlist.tracks.length > 0
	);

	let activeIndex = $derived($playlist?.currentIndex ?? -1);

	$effect(() => {
		if (!visible || collapsed || activeIndex < 0 || !listContainer) return;
		void scrollActiveIntoView(activeIndex);
	});

	function toggleCollapsed(): void {
		collapsed = !collapsed;
	}

	async function scrollActiveIntoView(index: number): Promise<void> {
		await tick();
		if (!listContainer) return;
		const el = listContainer.querySelector(`[data-track-index="${index}"]`);
		if (el) {
			el.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
		}
	}

	async function handlePick(index: number): Promise<void> {
		const pl = $playlist;
		if (!pl) return;
		if (index === pl.currentIndex) return;
		const t = pl.tracks[index];
		if (!t || !t.youtubeUrl) return;
		switchingIndex = index;
		switchError = null;
		try {
			await playPlaylistTrack(pl, index);
		} catch (err) {
			switchError = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			switchingIndex = null;
		}
	}

	function formatDuration(secs: number | null): string {
		if (!secs || !Number.isFinite(secs) || secs <= 0) return '—';
		const total = Math.round(secs);
		const m = Math.floor(total / 60);
		const s = total % 60;
		return `${m}:${s.toString().padStart(2, '0')}`;
	}
</script>

{#if visible && $playlist}
	<div class="flex flex-col border-t border-base-300" aria-label="Playlist">
		<button
			type="button"
			class="flex items-center justify-between bg-base-200/60 px-3 py-1 text-left transition-colors hover:bg-base-200"
			onclick={toggleCollapsed}
			aria-expanded={!collapsed}
			aria-label={collapsed ? 'Expand playlist' : 'Collapse playlist'}
			title={collapsed ? 'Expand playlist' : 'Collapse playlist'}
		>
			<span class="flex min-w-0 items-center gap-2">
				<span class="text-xs font-semibold text-base-content/70">Playlist</span>
				<span class="badge badge-ghost badge-xs">{$playlist.tracks.length}</span>
				{#if $playlist.title}
					<span class="truncate text-xs text-base-content/60" title={$playlist.title}>
						{$playlist.title}
					</span>
				{/if}
			</span>
			<span class={classNames('transition-transform', { 'rotate-180': collapsed })}>
				<Icon name="delapouite/plain-arrow" size="0.75em" />
			</span>
		</button>
		{#if switchError}
			<div class="border-b border-base-300 bg-error/10 px-3 py-1 text-xs text-error">
				{switchError}
			</div>
		{/if}
		{#if !collapsed}
		<div bind:this={listContainer} class="max-h-64 overflow-y-auto scroll-smooth">
			<ol class="flex flex-col">
				{#each $playlist.tracks as track, index (index)}
					{@const isActive = index === activeIndex}
					{@const isSwitching = switchingIndex === index}
					{@const playable = !!track.youtubeUrl}
					<li>
						<button
							type="button"
							data-track-index={index}
							class={classNames(
								'flex w-full items-center gap-2 px-3 py-1.5 text-left text-xs transition-colors',
								{
									'bg-primary/10 font-semibold text-primary': isActive,
									'cursor-pointer hover:bg-base-200': !isActive && playable,
									'cursor-not-allowed opacity-50': !playable
								}
							)}
							disabled={!playable || isSwitching || (isActive && !isSwitching)}
							onclick={() => handlePick(index)}
							title={playable ? track.title : `${track.title} (no playable source)`}
						>
							<span class="w-6 shrink-0 text-right font-mono text-base-content/60">
								{track.position ?? index + 1}
							</span>
							<span class="min-w-0 flex-1 truncate">{track.title}</span>
							{#if isSwitching}
								<span class="loading loading-xs loading-spinner"></span>
							{:else if isActive}
								<span class="text-primary">
									<Icon name="guard13007/play-button" size="0.85em" title="Now playing" />
								</span>
							{/if}
							<span class="font-mono text-[10px] tabular-nums text-base-content/60">
								{formatDuration(track.durationSeconds)}
							</span>
						</button>
					</li>
				{/each}
			</ol>
		</div>
		{/if}
	</div>
{/if}
