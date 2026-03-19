<script lang="ts">
	import { cloudPeerService } from 'frontend/services/cloud-peer.service';
	import { cloudAdapter } from 'frontend/adapters/classes/cloud.adapter';

	const peerState = cloudPeerService.state;

	let svc = $derived($peerState);
	let peerIds = $derived(Object.keys(svc.peers));

	let selectedPeer = $state<string | null>(null);
	let selectedLibrary = $state<string | null>(null);

	let currentPeerData = $derived(selectedPeer ? svc.peers[selectedPeer] : null);
	let currentItems = $derived(
		selectedPeer && selectedLibrary ? (svc.peers[selectedPeer]?.items[selectedLibrary] ?? []) : []
	);

	function handleSelectPeer(peerId: string) {
		selectedPeer = peerId;
		selectedLibrary = null;
	}

	function handleSelectLibrary(peerId: string, libraryId: string) {
		selectedLibrary = libraryId;
		cloudPeerService.requestItems(peerId, libraryId);
	}
</script>

<div class="p-6">
	<h1 class="mb-6 text-2xl font-bold">Peer Cloud Libraries</h1>

	{#if peerIds.length === 0}
		<div class="py-20 text-center">
			<p class="text-lg text-base-content/60">No peers connected</p>
			<p class="mt-2 text-sm text-base-content/40">
				Connect to a signaling room to discover peers.
			</p>
		</div>
	{:else}
		<div class="flex gap-6">
			<div class="w-64 shrink-0">
				<h3 class="mb-3 text-sm font-semibold tracking-wide uppercase">Connected Peers</h3>
				<div class="space-y-1">
					{#each peerIds as peerId (peerId)}
						<button
							class="btn w-full justify-start btn-ghost btn-sm {selectedPeer === peerId
								? 'btn-active'
								: ''}"
							onclick={() => handleSelectPeer(peerId)}
						>
							{peerId.slice(0, 8)}...
						</button>
					{/each}
				</div>
			</div>

			<div class="flex-1">
				{#if selectedPeer && currentPeerData}
					<h3 class="mb-3 text-lg font-semibold">
						Libraries from {selectedPeer.slice(0, 8)}...
					</h3>

					{#if currentPeerData.libraries.length === 0}
						<p class="text-base-content/60">This peer has no cloud libraries.</p>
					{:else}
						<div class="grid grid-cols-1 gap-3 md:grid-cols-2">
							{#each currentPeerData.libraries as lib (lib.id)}
								<button
									class="card cursor-pointer bg-base-100 shadow-sm transition-shadow hover:shadow-md {selectedLibrary ===
									lib.id
										? 'ring-2 ring-primary'
										: ''}"
									onclick={() => handleSelectLibrary(selectedPeer!, lib.id)}
								>
									<div class="card-body p-4">
										<h4 class="font-medium">{lib.name}</h4>
										<div class="flex gap-2">
											<span class="badge badge-ghost badge-sm">{lib.kind}</span>
											<span class="badge badge-sm badge-neutral">{lib.itemCount} items</span>
										</div>
									</div>
								</button>
							{/each}
						</div>
					{/if}

					{#if selectedLibrary && currentItems.length > 0}
						<div class="mt-6">
							<h4 class="mb-2 text-sm font-semibold">Items</h4>
							<div class="overflow-x-auto">
								<table class="table table-sm">
									<thead>
										<tr>
											<th>Filename</th>
											<th>Extension</th>
											<th>Size</th>
										</tr>
									</thead>
									<tbody>
										{#each currentItems as item (item.id)}
											<tr>
												<td>{item.filename}</td>
												<td>
													<span class="badge badge-ghost badge-xs">{item.extension}</span>
												</td>
												<td class="text-xs text-base-content/60">
													{cloudAdapter.formatBytes(item.sizeBytes)}
												</td>
											</tr>
										{/each}
									</tbody>
								</table>
							</div>
						</div>
					{/if}
				{:else}
					<p class="text-base-content/60">Select a peer to browse their cloud libraries.</p>
				{/if}
			</div>
		</div>
	{/if}
</div>
