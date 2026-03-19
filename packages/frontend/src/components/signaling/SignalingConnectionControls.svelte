<script lang="ts">
	import classNames from 'classnames';
	import { onMount } from 'svelte';
	import { apiUrl } from 'frontend/lib/api-base';
	import { signalingChatService } from 'frontend/services/signaling-chat.service';
	import { signalingAdapter } from 'frontend/adapters/classes/signaling.adapter';
	import type { SignalingServerTarget, SignalingServerStatus } from 'frontend/types/signaling.type';

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
		if (serverTarget === 'dev') return signalingAdapter.resolveLocalUrl(serverStatus.devUrl);
		const server = serverStatus.servers.find((s) => s.id === serverTarget);
		return server?.url ?? '';
	}

	function isServerAvailable(): boolean {
		if (!serverStatus) return false;
		if (serverTarget === 'dev') return serverStatus.devAvailable;
		const server = serverStatus.servers.find((s) => s.id === serverTarget);
		return server?.available ?? false;
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

<div class="card flex flex-col bg-base-200">
	<div class="card-body gap-4 p-4">
		<div class="flex items-center justify-between">
			<h2 class="card-title text-base">Connection</h2>
			<span
				class={classNames('badge badge-sm', signalingAdapter.phaseBadgeClass($chatStore.phase))}
			>
				{signalingAdapter.phaseLabel($chatStore.phase)}
			</span>
		</div>

		<!-- Server Selection -->
		<select
			class="select-bordered select select-sm"
			bind:value={serverTarget}
			disabled={$chatStore.phase !== 'disconnected'}
		>
			<option value="dev">Local</option>
			{#if serverStatus}
				{#each serverStatus.servers as server (server.id)}
					<option value={server.id}>{server.name}</option>
				{/each}
			{/if}
		</select>

		<!-- Server URL display -->
		{#if serverStatus}
			<div class="flex items-center gap-2">
				<span
					class={classNames('h-2 w-2 rounded-full', {
						'bg-success': isServerAvailable(),
						'bg-error': !isServerAvailable()
					})}
				></span>
				<span class="truncate font-mono text-xs text-base-content/60"
					>{getServerUrl() || 'Not configured'}</span
				>
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
				class="input-bordered input input-sm font-mono"
				placeholder="Enter room ID..."
				bind:value={roomId}
				onkeydown={handleKeydown}
				disabled={$chatStore.phase !== 'disconnected'}
			/>
		</div>

		<!-- Connect / Disconnect -->
		{#if $chatStore.phase === 'disconnected' || $chatStore.phase === 'error'}
			<button
				class="btn btn-sm btn-primary"
				disabled={!getServerUrl() || !roomId.trim() || !isServerAvailable()}
				onclick={handleConnect}
			>
				Connect
			</button>
		{:else}
			<button class="btn btn-outline btn-sm btn-error" onclick={handleDisconnect}>
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
				<h3 class="mb-1 text-xs font-semibold tracking-wide text-base-content/50 uppercase">
					Peers ({$chatStore.peerIds.length})
				</h3>
				<div class="flex flex-wrap gap-1">
					{#each $chatStore.peerIds as peerId (peerId)}
						<span class="badge badge-outline font-mono badge-sm">
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
