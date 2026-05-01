<script lang="ts">
	import classNames from 'classnames';
	import { firkinPlaybackService } from '$services/firkin-playback.service';
	import { isPlayableFile } from '$services/firkin-stream.service';
	import type { FirkinFile } from '$types/firkin.type';

	interface Props {
		onPlayFile?: (file: FirkinFile) => void | Promise<void>;
	}

	let { onPlayFile }: Props = $props();

	const state = firkinPlaybackService.state;
</script>

{#if $state.firkin}
	<section class="flex flex-col gap-2 rounded-box border border-base-content/10 bg-base-300 p-3">
		<header class="flex items-center justify-between gap-2">
			<h3 class="truncate text-sm font-semibold" title={$state.firkin.title}>
				{$state.firkin.title}
			</h3>
			<button
				type="button"
				class="btn btn-ghost btn-xs"
				onclick={() => firkinPlaybackService.clear()}
				aria-label="Close playback panel"
			>
				×
			</button>
		</header>
		{#if $state.files.length === 0}
			<p class="text-xs text-base-content/60">No IPFS files for this firkin yet.</p>
		{:else}
			<ul class="flex flex-col gap-1">
				{#each $state.files as file (file.value)}
					{@const playable = !!onPlayFile && isPlayableFile(file)}
					<li
						class={classNames(
							'flex flex-col gap-0.5 rounded border border-base-content/10 bg-base-100 px-2 py-1',
							{ 'cursor-pointer hover:bg-base-200': playable }
						)}
					>
						{#if playable}
							<button
								type="button"
								class="flex flex-col items-start gap-0.5 text-left"
								onclick={() => onPlayFile?.(file)}
							>
								<span
									class="flex w-full items-center gap-1 truncate text-xs font-medium"
									title={file.title ?? file.value}
								>
									<svg
										xmlns="http://www.w3.org/2000/svg"
										viewBox="0 0 24 24"
										fill="currentColor"
										stroke="none"
										class="h-3 w-3 shrink-0 text-primary"
										aria-hidden="true"
									>
										<polygon points="6 4 20 12 6 20 6 4" />
									</svg>
									<span class="truncate">{file.title ?? file.value}</span>
								</span>
								<span
									class="truncate font-mono text-[10px] text-base-content/60"
									title={file.value}
								>
									{file.value}
								</span>
							</button>
						{:else}
							<span class="truncate text-xs font-medium" title={file.title ?? file.value}>
								{file.title ?? file.value}
							</span>
							<span class="truncate font-mono text-[10px] text-base-content/60" title={file.value}>
								{file.value}
							</span>
						{/if}
					</li>
				{/each}
			</ul>
		{/if}
	</section>
{/if}
