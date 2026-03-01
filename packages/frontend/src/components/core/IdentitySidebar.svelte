<script lang="ts">
	import classNames from 'classnames';
	import { onMount } from 'svelte';
	import { identityService } from '$services/identity.service';
	import { identityAdapter } from '$adapters/classes/identity.adapter';
	import { playerService } from '$services/player.service';
	import { signalingAdapter } from '$adapters/classes/signaling.adapter';
	import { signalingChatService } from '$services/signaling-chat.service';
	import { sidebarService } from '$services/sidebar.service';
	import { mediaDetailService } from '$services/media-detail.service';
	import PlayerVideo from '$components/player/PlayerVideo.svelte';
	import LyricsPanel from '$components/player/LyricsPanel.svelte';
	import MediaDetail from '$components/media/MediaDetail.svelte';
	import DownloadsSummary from '$components/downloads/DownloadsSummary.svelte';
	import type { SignalingServerStatus } from '$types/signaling.type';
	import type { SidebarWidthMode } from '$types/sidebar.type';

	interface Props {
		classes?: string;
	}

	let { classes = '' }: Props = $props();

	const identityState = identityService.state;
	const playerState = playerService.state;
	const chatState = signalingChatService.state;
	const sidebarSettings = sidebarService.store;
	const mediaDetailSelection = mediaDetailService.store;

	let signalingStatus = $state<SignalingServerStatus | null>(null);

	onMount(async () => {
		try {
			const res = await fetch('/api/signaling/status');
			if (res.ok) signalingStatus = await res.json();
		} catch {
			// Ignore
		}
	});

	let serverAvailable = $derived(
		signalingStatus ? signalingStatus.devAvailable || signalingStatus.deployedAvailable : false
	);

	let serverUrl = $derived(
		signalingStatus
			? signalingStatus.deployedAvailable
				? signalingStatus.partyUrl
				: signalingStatus.devUrl
			: null
	);

	const widthClasses: Record<SidebarWidthMode, string> = {
		wide: 'w-[80vw]',
		default: 'w-128',
		narrow: 'w-85'
	};

	let wrapperClasses = $derived(
		classNames(
			'hidden lg:flex flex-col bg-base-200 border-l border-base-300 p-4 overflow-y-auto',
			widthClasses[$sidebarSettings.widthMode],
			classes
		)
	);

	const widthModes: { mode: SidebarWidthMode; label: string }[] = [
		{ mode: 'wide', label: 'Wide' },
		{ mode: 'default', label: 'Default' },
		{ mode: 'narrow', label: 'Narrow' }
	];
</script>

<aside class={wrapperClasses}>
	<div class="mb-4 flex justify-center">
		<div class="join">
			{#each widthModes as { mode, label }}
				<button
					class={classNames('btn btn-xs join-item', {
						'btn-active': $sidebarSettings.widthMode === mode
					})}
					onclick={() => sidebarService.setWidthMode(mode)}
				>
					{label}
				</button>
			{/each}
		</div>
	</div>

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

	<div class="mt-4 border-t border-base-300 pt-4">
		<div class="mb-2 flex items-center justify-between">
			<h2 class="text-sm font-semibold uppercase tracking-wide text-base-content/50">
				Signaling Server
			</h2>
			{#if signalingStatus}
				<span
					class={classNames('h-2 w-2 rounded-full', {
						'bg-success': serverAvailable,
						'bg-error': !serverAvailable
					})}
				></span>
			{/if}
		</div>

		{#if !signalingStatus}
			<p class="text-xs opacity-50">Loading...</p>
		{:else}
			<div class="flex flex-col gap-2">
				<span class="truncate font-mono text-xs text-base-content/60" title={serverUrl ?? ''}>
					{serverUrl || 'Not configured'}
				</span>
				<div class="flex items-center gap-2">
					<span
						class={classNames(
							'badge badge-sm',
							signalingAdapter.playerConnectionBadgeClass($playerState.connectionState)
						)}
					>
						{signalingAdapter.playerConnectionLabel($playerState.connectionState)}
					</span>
				</div>

				<!-- Server WebRTC endpoint (streaming worker) -->
				{#if $playerState.localPeerId || $playerState.remotePeerId}
					<div class="flex flex-col gap-1">
						<span class="text-xs font-semibold text-base-content/50">Stream Peers</span>
						{#if $playerState.localPeerId}
							<div class="flex items-center gap-1">
								<span class="text-xs text-base-content/40">You:</span>
								<span class="font-mono text-xs text-base-content/60">
									{signalingAdapter.shortAddress($playerState.localPeerId)}
								</span>
							</div>
						{/if}
						{#if $playerState.remotePeerId}
							<div class="flex items-center gap-1">
								<span class="text-xs text-base-content/40">Worker:</span>
								<span class="badge badge-outline badge-sm font-mono">
									{signalingAdapter.shortAddress($playerState.remotePeerId)}
								</span>
							</div>
						{/if}
					</div>
				{/if}

				<!-- Browser WebRTC endpoint (signaling chat peers) -->
				{#if $chatState.phase !== 'disconnected'}
					<div class="flex flex-col gap-1">
						<span class="text-xs font-semibold text-base-content/50">
							Chat Peers ({$chatState.peerIds.length})
						</span>
						{#if $chatState.localPeerId}
							<div class="flex items-center gap-1">
								<span class="text-xs text-base-content/40">You:</span>
								<span class="font-mono text-xs text-base-content/60">
									{signalingAdapter.shortAddress($chatState.localPeerId)}
								</span>
							</div>
						{/if}
						{#if $chatState.peerIds.length > 0}
							<div class="flex flex-wrap gap-1">
								{#each $chatState.peerIds as peerId (peerId)}
									<span class="badge badge-outline badge-sm font-mono">
										{signalingAdapter.shortAddress(peerId)}
									</span>
								{/each}
							</div>
						{:else}
							<span class="text-xs opacity-40">No peers</span>
						{/if}
					</div>
				{/if}
			</div>
		{/if}
	</div>

	<div class="mt-4 border-t border-base-300 pt-4">
		<DownloadsSummary />
	</div>

	{#if $mediaDetailSelection}
		<div class="mt-4 border-t border-base-300 pt-4">
			<MediaDetail
				selection={$mediaDetailSelection}
				onclose={() => mediaDetailService.clear()}
			/>
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
			{#if $playerState.currentFile.mode === 'audio'}
				<div class="mt-2">
					<LyricsPanel
						currentFile={$playerState.currentFile}
						positionSecs={$playerState.positionSecs}
						on:seek={(e) => playerService.seek(e.detail.positionSecs)}
					/>
				</div>
			{/if}
		</div>
	{/if}
</aside>
