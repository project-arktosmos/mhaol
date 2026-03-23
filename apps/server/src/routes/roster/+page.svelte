<script lang="ts">
	import classNames from 'classnames';
	import { rosterService } from 'ui-lib/services/roster.service';
	import { identityAdapter } from 'ui-lib/adapters/classes/identity.adapter';
	import { signalingChatService } from 'ui-lib/services/signaling-chat.service';
	import { toastService } from 'ui-lib/services/toast.service';
	import type { RosterPeerStatus } from 'ui-lib/types/roster.type';

	const rosterStore = rosterService.state;
	const chatStore = signalingChatService.state;

	let confirmingRemove = $state<string | null>(null);

	function statusDotClass(status: RosterPeerStatus): string {
		const map: Record<RosterPeerStatus, string> = {
			online: 'bg-success',
			checking: 'bg-warning animate-pulse',
			offline: 'bg-base-content/20'
		};
		return map[status];
	}

	function statusLabel(status: RosterPeerStatus): string {
		const labels: Record<RosterPeerStatus, string> = {
			online: 'Online',
			checking: 'Checking...',
			offline: 'Offline'
		};
		return labels[status];
	}

	function handleRefresh() {
		rosterService.refresh();
	}

	async function handleRemove(address: string) {
		await rosterService.removeEntry(address);
		confirmingRemove = null;
	}

	function handleConnect(address: string) {
		const peerId = findPeerIdForAddress(address);
		if (peerId) {
			signalingChatService.connectToPeer(peerId);
		} else {
			toastService.warning('Peer is not in the signaling room');
		}
	}

	function findPeerIdForAddress(address: string): string | null {
		const peers = $chatStore.roomPeers ?? [];
		const match = peers.find(
			(p) => p.peer_id.toLowerCase() === address.toLowerCase()
		);
		return match?.peer_id ?? null;
	}
</script>

<div class="container mx-auto p-4">
	<div class="mb-6 flex items-center justify-between">
		<div>
			<h1 class="text-3xl font-bold">Roster</h1>
			<p class="text-sm opacity-70">
				Manage WebRTC passport contacts and their signaling status.
			</p>
		</div>
		<button
			class="btn btn-primary btn-sm"
			onclick={handleRefresh}
			disabled={$rosterStore.loading}
		>
			{#if $rosterStore.loading}
				<span class="loading loading-xs loading-spinner"></span>
			{:else}
				Refresh
			{/if}
		</button>
	</div>

	{#if $rosterStore.error}
		<div class="mb-4 alert alert-error">
			<span>{$rosterStore.error}</span>
			<button
				class="btn btn-ghost btn-sm"
				onclick={() => rosterService.state.update((s) => ({ ...s, error: null }))}
			>
				x
			</button>
		</div>
	{/if}

	{#if $rosterStore.loading && $rosterStore.entries.length === 0}
		<div class="flex justify-center py-12">
			<span class="loading loading-lg loading-spinner"></span>
		</div>
	{:else if $rosterStore.entries.length === 0}
		<div class="rounded-lg bg-base-200 p-8 text-center">
			<p class="opacity-50">No contacts in roster yet.</p>
			<p class="mt-1 text-sm opacity-40">
				Contacts are added when peers exchange passports via signaling.
			</p>
		</div>
	{:else}
		<div class="flex flex-col gap-2">
			{#each $rosterStore.entries as entry (entry.address)}
				<div class="card bg-base-200">
					<div class="card-body flex-row items-center gap-3 p-4">
						<span
							class={classNames(
								'h-3 w-3 shrink-0 rounded-full',
								statusDotClass(entry.status)
							)}
						></span>
						<div class="min-w-0 flex-1">
							<div class="flex items-center gap-2">
								<span class="font-mono text-sm font-semibold">{entry.name}</span>
								{#if entry.instanceType}
									<span class="badge badge-outline badge-xs">{entry.instanceType}</span>
								{/if}
							</div>
							<code class="block break-all text-xs opacity-50">
								{identityAdapter.shortAddress(entry.address)}
							</code>
						</div>
						<div class="flex items-center gap-2">
							<span
								class={classNames('badge badge-sm', {
									'badge-success': entry.status === 'online',
									'badge-warning': entry.status === 'checking',
									'badge-ghost': entry.status === 'offline'
								})}
							>
								{statusLabel(entry.status)}
							</span>
							{#if entry.status === 'online'}
								<button
									class="btn btn-ghost btn-xs"
									onclick={() => handleConnect(entry.address)}
									title="Connect to peer"
								>
									Connect
								</button>
							{/if}
							{#if confirmingRemove === entry.address}
								<button
									class="btn btn-error btn-xs"
									onclick={() => handleRemove(entry.address)}
								>
									Confirm
								</button>
								<button
									class="btn btn-ghost btn-xs"
									onclick={() => (confirmingRemove = null)}
								>
									Cancel
								</button>
							{:else}
								<button
									class="btn btn-ghost btn-xs text-error"
									onclick={() => (confirmingRemove = entry.address)}
									title="Remove from roster"
								>
									Remove
								</button>
							{/if}
						</div>
					</div>
				</div>
			{/each}
		</div>

		<div class="mt-4 text-xs text-base-content/40">
			{$rosterStore.entries.filter((e) => e.status === 'online').length} of
			{$rosterStore.entries.length} online
		</div>
	{/if}
</div>
