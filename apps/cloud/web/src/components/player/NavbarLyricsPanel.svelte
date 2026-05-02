<script lang="ts">
	import classNames from 'classnames';
	import { tick } from 'svelte';
	import { playerService } from '$services/player.service';

	const playerState = playerService.state;
	const playerDisplayMode = playerService.displayMode;

	let container: HTMLDivElement | null = $state(null);
	let dismissedFileId: string | null = $state(null);
	let anchorRight: number = $state(8);
	let anchorWidth: number = $state(360);

	let visible = $derived(
		$playerDisplayMode === 'navbar' &&
			$playerState.currentFile !== null &&
			Array.isArray($playerState.syncedLyrics) &&
			$playerState.syncedLyrics.length > 0 &&
			$playerState.currentFile.id !== dismissedFileId
	);

	let lines = $derived($playerState.syncedLyrics ?? []);
	let position = $derived($playerState.positionSecs);

	let currentLineIndex = $derived.by(() => {
		const ls = lines;
		if (ls.length === 0) return -1;
		let idx = -1;
		for (let i = 0; i < ls.length; i++) {
			if (ls[i].time <= position) idx = i;
			else break;
		}
		return idx;
	});

	function measureAnchor(): void {
		if (typeof document === 'undefined') return;
		const el = document.querySelector<HTMLElement>('[data-navbar-audio-player]');
		if (!el) return;
		const rect = el.getBoundingClientRect();
		const viewportWidth = window.innerWidth;
		anchorRight = Math.max(viewportWidth - rect.right, 0);
		anchorWidth = Math.max(rect.width, 320);
	}

	$effect(() => {
		if (!visible) return;
		void tick().then(measureAnchor);
		const onResize = () => measureAnchor();
		window.addEventListener('resize', onResize);
		const audioEl = document.querySelector<HTMLElement>('[data-navbar-audio-player]');
		const ro =
			audioEl && typeof ResizeObserver !== 'undefined' ? new ResizeObserver(measureAnchor) : null;
		if (audioEl && ro) ro.observe(audioEl);
		return () => {
			window.removeEventListener('resize', onResize);
			ro?.disconnect();
		};
	});

	$effect(() => {
		if (!visible || currentLineIndex < 0 || !container) return;
		void scrollToCurrentLine(currentLineIndex);
	});

	async function scrollToCurrentLine(index: number): Promise<void> {
		await tick();
		if (!container) return;
		const lineEl = container.querySelector(`[data-line-index="${index}"]`);
		if (lineEl) {
			lineEl.scrollIntoView({ behavior: 'smooth', block: 'center' });
		}
	}

	function handleLineClick(time: number): void {
		// `seek()` only moves the store; the navbar `<video>` element doesn't
		// observe positionSecs writes, so reach for the playing media element
		// directly to actually move the playhead.
		const el = document.querySelector<HTMLVideoElement>('video');
		if (el && Number.isFinite(time)) {
			el.currentTime = time;
		}
		playerService.seek(time);
	}

	function handleDismiss(): void {
		const id = $playerState.currentFile?.id ?? null;
		dismissedFileId = id;
	}
</script>

{#if visible}
	<div
		class="pointer-events-none fixed top-16 z-40"
		style:right="{anchorRight}px"
		style:width="{anchorWidth}px"
		aria-label="Synced lyrics"
	>
		<div
			class="pointer-events-auto flex flex-col overflow-hidden rounded-b-lg border border-t-0 border-base-300 bg-base-100 shadow-lg"
		>
			<div
				class="flex items-center justify-between border-b border-base-300 bg-base-200/60 px-3 py-1"
			>
				<div class="flex min-w-0 items-center gap-2">
					<span class="text-xs font-semibold text-base-content/70">Lyrics</span>
					<span class="badge badge-xs badge-primary">Synced</span>
				</div>
				<button
					type="button"
					class="btn btn-ghost btn-xs"
					onclick={handleDismiss}
					aria-label="Hide lyrics"
					title="Hide lyrics"
				>
					✕
				</button>
			</div>
			<div bind:this={container} class="max-h-64 overflow-y-auto scroll-smooth px-3 py-2">
				<div class="space-y-1 py-2">
					{#each lines as line, index (index)}
						<button
							type="button"
							data-line-index={index}
							class={classNames(
								'w-full cursor-pointer rounded px-2 py-1 text-left text-sm transition-all duration-200',
								{
									'bg-primary font-semibold text-primary-content': index === currentLineIndex,
									'text-base-content/60 hover:bg-base-300/50': index !== currentLineIndex
								}
							)}
							onclick={() => handleLineClick(line.time)}
						>
							{#if line.text}
								{line.text}
							{:else}
								<span class="text-base-content/20">…</span>
							{/if}
						</button>
					{/each}
				</div>
			</div>
		</div>
	</div>
{/if}
