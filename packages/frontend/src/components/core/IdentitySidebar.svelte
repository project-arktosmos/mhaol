<script lang="ts">
	import classNames from 'classnames';
	import { identityService } from '$services/identity.service';
	import { identityAdapter } from '$adapters/classes/identity.adapter';
	import { playerService } from '$services/player.service';
	import PlayerVideo from '$components/player/PlayerVideo.svelte';

	interface Props {
		classes?: string;
	}

	let { classes = '' }: Props = $props();

	const identityState = identityService.state;
	const playerState = playerService.state;

	let wrapperClasses = $derived(
		classNames(
			'hidden lg:flex flex-col w-128 bg-base-200 border-l border-base-300 p-4 overflow-y-auto',
			classes
		)
	);
</script>

<aside class={wrapperClasses}>
	<h2 class="mb-3 text-sm font-semibold uppercase tracking-wide text-base-content/50">
		Identities
	</h2>

	{#if $identityState.loading}
		<div class="flex justify-center py-4">
			<span class="loading loading-spinner loading-sm"></span>
		</div>
	{:else if $identityState.error}
		<p class="text-xs text-error">{$identityState.error}</p>
	{:else if $identityState.identities.length === 0}
		<p class="text-xs opacity-50">No identities</p>
	{:else}
		<div class="flex flex-col gap-2">
			{#each $identityState.identities as identity (identity.name)}
				<div class="rounded-lg bg-base-100 p-3">
					<div class="font-mono text-xs font-semibold">{identity.name}</div>
					<div class="mt-1 font-mono text-xs opacity-60">
						{identityAdapter.shortAddress(identity.address)}
					</div>
				</div>
			{/each}
		</div>
	{/if}

	{#if $playerState.currentFile}
		<div class="mt-4 border-t border-base-300 pt-4">
			<div class="mb-2 flex items-center justify-between">
				<h2 class="text-sm font-semibold uppercase tracking-wide text-base-content/50">
					Now Playing
				</h2>
				<button
					class="btn btn-ghost btn-xs btn-square"
					onclick={() => playerService.stop()}
					aria-label="Close player"
				>
					&times;
				</button>
			</div>
			<p class="mb-2 truncate text-xs opacity-60" title={$playerState.currentFile.name}>
				{$playerState.currentFile.name}
			</p>
			<PlayerVideo
				file={$playerState.currentFile}
				connectionState={$playerState.connectionState}
				positionSecs={$playerState.positionSecs}
				durationSecs={$playerState.durationSecs}
			/>
		</div>
	{/if}
</aside>
