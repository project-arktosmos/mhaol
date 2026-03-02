<script lang="ts">
	import classNames from 'classnames';
	import { onMount } from 'svelte';
	import { apiUrl } from '$lib/api-base';
	import { signalingChatService } from '$services/signaling-chat.service';
	import { signalingAdapter } from '$adapters/classes/signaling.adapter';
	import type { SignalingServerTarget, SignalingServerStatus } from '$types/signaling.type';

	const chatStore = signalingChatService.state;

	let serverTarget = $state<SignalingServerTarget>('dev');
	let roomId = $state('test-room');
	let serverStatus = $state<SignalingServerStatus | null>(null);

	onMount(async () => {
		try {
			const res = await fetch(apiUrl('/api/signaling/status'));
			if (res.ok) serverStatus = await res.json();
		} catch {
			// Ignore
		}
	});

	function getServerUrl(): string {
		if (!serverStatus) return '';
		return serverTarget === 'dev' ? serverStatus.devUrl : serverStatus.partyUrl;
	}

	function isServerAvailable(): boolean {
		if (!serverStatus) return false;
		return serverTarget === 'dev' ? serverStatus.devAvailable : serverStatus.deployedAvailable;
	}

	function handleConnect() {
		const url = getServerUrl();
		if (!url || !roomId.trim()) return;
		signalingChatService.connect(url, roomId.trim(), serverTarget);
	}

	function handleDisconnect() {
		signalingChatService.disconnect();
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter' && $chatStore.phase === 'disconnected') {
			handleConnect();
		}
	}
</script>

<div class="card bg-base-200 flex flex-col">
	<div class="card-body gap-4 p-4">
		<div class="flex items-center justify-between">
			<h2 class="card-title text-base">Connection</h2>
			<span class={classNames('badge badge-sm', signalingAdapter.phaseBadgeClass($chatStore.phase))}>
				{signalingAdapter.phaseLabel($chatStore.phase)}
			</span>
		</div>

		<!-- Server Target Tabs -->
		<div class="tabs tabs-box">
			<button
				class={classNames('tab', { 'tab-active': serverTarget === 'dev' })}
				onclick={() => (serverTarget = 'dev')}
				disabled={$chatStore.phase !== 'disconnected'}
			>
				Dev Server
			</button>
			<button
				class={classNames('tab', { 'tab-active': serverTarget === 'deployed' })}
				onclick={() => (serverTarget = 'deployed')}
				disabled={$chatStore.phase !== 'disconnected'}
			>
				Deployed
			</button>
		</div>

		<!-- Server URL display -->
		{#if serverStatus}
			<div class="flex items-center gap-2">
				<span
					class={classNames('h-2 w-2 rounded-full', {
						'bg-success': isServerAvailable(),
						'bg-error': !isServerAvailable()
					})}
				></span>
				<span class="truncate font-mono text-xs text-base-content/60">{getServerUrl() || 'Not configured'}</span>
			</div>
		{/if}

		<!-- Room ID -->
		<div class="form-control">
			<label class="label" for="room-id-input">
				<span class="label-text text-sm">Room ID</span>
			</label>
			<input
				id="room-id-input"
				type="text"
				class="input input-bordered input-sm font-mono"
				placeholder="Enter room ID..."
				bind:value={roomId}
				onkeydown={handleKeydown}
				disabled={$chatStore.phase !== 'disconnected'}
			/>
		</div>

		<!-- Connect / Disconnect -->
		{#if $chatStore.phase === 'disconnected' || $chatStore.phase === 'error'}
			<button
				class="btn btn-primary btn-sm"
				disabled={!getServerUrl() || !roomId.trim() || !isServerAvailable()}
				onclick={handleConnect}
			>
				Connect
			</button>
		{:else}
			<button class="btn btn-error btn-sm btn-outline" onclick={handleDisconnect}>
				Disconnect
			</button>
		{/if}

		<!-- Error display -->
		{#if $chatStore.error}
			<p class="text-sm text-error">{$chatStore.error}</p>
		{/if}

		<!-- Connected Peers -->
		{#if $chatStore.peerIds.length > 0}
			<div>
				<h3 class="mb-1 text-xs font-semibold uppercase tracking-wide text-base-content/50">
					Peers ({$chatStore.peerIds.length})
				</h3>
				<div class="flex flex-wrap gap-1">
					{#each $chatStore.peerIds as peerId (peerId)}
						<span class="badge badge-outline badge-sm font-mono">
							{signalingAdapter.shortAddress(peerId)}
						</span>
					{/each}
				</div>
			</div>
		{/if}

		<!-- Local Peer ID -->
		{#if $chatStore.localPeerId}
			<p class="text-xs text-base-content/40">
				You: <span class="font-mono">{signalingAdapter.shortAddress($chatStore.localPeerId)}</span>
			</p>
		{/if}
	</div>
</div>
