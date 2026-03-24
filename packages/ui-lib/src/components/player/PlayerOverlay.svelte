<script lang="ts">
	import classNames from 'classnames';
	import { playerService } from 'ui-lib/services/player.service';
	import PlayerVideo from 'ui-lib/components/player/PlayerVideo.svelte';

	const playerState = playerService.state;
	const playerDisplayMode = playerService.displayMode;

	let minimized = $state(false);
</script>

{#if $playerState.currentFile && $playerDisplayMode !== 'inline'}
	<div
		class={classNames({
			'fixed inset-0 z-50 flex flex-col bg-black': $playerDisplayMode === 'fullscreen',
			'fixed right-4 bottom-4 z-40 w-96 overflow-hidden rounded-lg bg-black shadow-2xl':
				$playerDisplayMode !== 'fullscreen' && !minimized,
			'fixed right-4 bottom-4 z-40 overflow-hidden rounded-lg bg-black shadow-2xl':
				$playerDisplayMode !== 'fullscreen' && minimized
		})}
	>
		<div
			class={classNames('flex items-center justify-between px-2 py-1', {
				'p-3': $playerDisplayMode === 'fullscreen'
			})}
		>
			<p
				class={classNames('min-w-0 truncate font-semibold text-white', {
					'text-xs': $playerDisplayMode !== 'fullscreen',
					'text-sm': $playerDisplayMode === 'fullscreen'
				})}
				title={$playerState.currentFile.name}
			>
				{$playerState.currentFile.name}
			</p>
			<div class="flex shrink-0 items-center gap-1">
				{#if $playerDisplayMode === 'fullscreen'}
					<button
						class="btn btn-square text-white btn-ghost btn-sm"
						onclick={() => {
							playerService.setDisplayMode('sidebar');
							minimized = false;
						}}
						aria-label="Minimize player"
						title="Minimize player"
					>
						<svg
							xmlns="http://www.w3.org/2000/svg"
							class="h-4 w-4"
							fill="none"
							viewBox="0 0 24 24"
							stroke="currentColor"
							stroke-width="2"
						>
							<path stroke-linecap="round" stroke-linejoin="round" d="M18 8L14 12L18 16" /><rect
								x="3"
								y="3"
								width="18"
								height="18"
								rx="2"
							/><line x1="14" y1="3" x2="14" y2="21" />
						</svg>
					</button>
				{:else}
					<button
						class="btn btn-square text-white btn-ghost btn-xs"
						onclick={() => (minimized = !minimized)}
						aria-label={minimized ? 'Expand player' : 'Minimize player'}
						title={minimized ? 'Expand player' : 'Minimize player'}
					>
						<svg
							xmlns="http://www.w3.org/2000/svg"
							class="h-3.5 w-3.5"
							fill="none"
							viewBox="0 0 24 24"
							stroke="currentColor"
							stroke-width="2"
						>
							{#if minimized}
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									d="M4 8V4h4M20 8V4h-4M4 16v4h4M20 16v4h-4"
								/>
							{:else}
								<path stroke-linecap="round" stroke-linejoin="round" d="M19 9l-7 7-7-7" />
							{/if}
						</svg>
					</button>
					<button
						class="btn btn-square text-white btn-ghost btn-xs"
						onclick={() => playerService.setDisplayMode('fullscreen')}
						aria-label="Fullscreen player"
						title="Fullscreen player"
					>
						<svg
							xmlns="http://www.w3.org/2000/svg"
							class="h-3.5 w-3.5"
							fill="none"
							viewBox="0 0 24 24"
							stroke="currentColor"
							stroke-width="2"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								d="M4 8V4h4M20 8V4h-4M4 16v4h4M20 16v4h-4"
							/>
						</svg>
					</button>
				{/if}
				<button
					class={classNames('btn btn-square text-white btn-ghost', {
						'btn-sm': $playerDisplayMode === 'fullscreen',
						'btn-xs': $playerDisplayMode !== 'fullscreen'
					})}
					onclick={() => playerService.stop()}
					aria-label="Close player"
				>
					&times;
				</button>
			</div>
		</div>
		{#if !minimized || $playerDisplayMode === 'fullscreen'}
			<div class={$playerDisplayMode === 'fullscreen' ? 'min-h-0 flex-1' : ''}>
				<PlayerVideo
					file={$playerState.currentFile}
					connectionState={$playerState.connectionState}
					positionSecs={$playerState.positionSecs}
					durationSecs={$playerState.durationSecs}
					buffering={$playerState.buffering}
					fullscreen={$playerDisplayMode === 'fullscreen'}
				/>
			</div>
		{/if}
	</div>
{/if}
