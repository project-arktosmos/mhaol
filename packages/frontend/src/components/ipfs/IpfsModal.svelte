<script lang="ts">
	import Modal from '$components/core/Modal.svelte';
	import { ipfsModalService } from '$services/ipfs-modal.service';
	import { ipfsService } from '$lib/ipfs.service';

	const pinsStore = ipfsService.state;
	const modalStore = ipfsModalService.store;
	let firstOpenSeen = false;

	$effect(() => {
		if ($modalStore.open && !firstOpenSeen) {
			firstOpenSeen = true;
			ipfsService.refresh();
		}
	});

	function close() {
		ipfsModalService.close();
	}

	function formatBytes(bytes: number): string {
		if (bytes < 1024) return `${bytes} B`;
		const units = ['KB', 'MB', 'GB', 'TB'];
		let value = bytes / 1024;
		let i = 0;
		while (value >= 1024 && i < units.length - 1) {
			value /= 1024;
			i++;
		}
		return `${value.toFixed(value >= 100 ? 0 : value >= 10 ? 1 : 2)} ${units[i]}`;
	}

	function formatDate(value: string): string {
		try {
			return new Date(value).toLocaleString();
		} catch {
			return value;
		}
	}
</script>

<Modal open={$modalStore.open} maxWidth="max-w-6xl" onclose={close}>
	<div class="flex flex-col gap-6">
		<header class="flex items-center justify-between gap-4">
			<div>
				<h2 class="text-2xl font-bold">IPFS</h2>
				<p class="text-sm text-base-content/60">
					Audio, video, and image files discovered while scanning libraries are pinned to the
					embedded IPFS node. This page lists every pin recorded by the cloud server.
				</p>
			</div>
			<button
				class="btn btn-outline btn-sm"
				onclick={() => ipfsService.refresh()}
				disabled={$pinsStore.loading}
			>
				Refresh
			</button>
		</header>

		{#if $pinsStore.error}
			<div class="alert alert-error">
				<span>{$pinsStore.error}</span>
			</div>
		{/if}

		{#if $pinsStore.loading && $pinsStore.pins.length === 0}
			<p class="text-sm text-base-content/60">Loading…</p>
		{:else if $pinsStore.pins.length === 0}
			<p class="text-sm text-base-content/60">
				No pins yet. Run a library scan to pin its audio files.
			</p>
		{:else}
			<div class="overflow-x-auto rounded-box border border-base-content/10">
				<table class="table table-sm">
					<thead>
						<tr>
							<th>CID</th>
							<th>Path</th>
							<th class="w-32">MIME</th>
							<th class="w-24 text-right">Size</th>
							<th class="w-40">Pinned</th>
						</tr>
					</thead>
					<tbody>
						{#each $pinsStore.pins as pin (pin.id)}
							<tr>
								<td class="font-mono text-xs break-all">{pin.cid}</td>
								<td class="font-mono text-xs break-all">{pin.path}</td>
								<td class="font-mono text-xs">{pin.mime}</td>
								<td class="text-right text-xs">{formatBytes(pin.size)}</td>
								<td class="text-xs text-base-content/60">{formatDate(pin.created_at)}</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{/if}
	</div>
</Modal>
