<script lang="ts">
	import { ipfsConfigStore } from '$ipfs/config.svelte';

	interface Props {
		open: boolean;
		onClose: () => void;
	}
	let { open, onClose }: Props = $props();

	let bootstrapInput = $state('');
	let swarmKeyInput = $state('');
	let saved = $state(false);

	$effect(() => {
		if (open) {
			bootstrapInput = ipfsConfigStore.bootstrapMultiaddrs.join('\n');
			swarmKeyInput = ipfsConfigStore.swarmKey;
			saved = false;
		}
	});

	function handleSave() {
		const lines = bootstrapInput
			.split('\n')
			.map((l) => l.trim())
			.filter((l) => l.length > 0);
		ipfsConfigStore.update({
			bootstrapMultiaddrs: lines,
			swarmKey: swarmKeyInput.trim() + (swarmKeyInput.endsWith('\n') ? '' : '\n')
		});
		saved = true;
	}
</script>

{#if open}
	<div class="modal-open modal">
		<div class="modal-box max-w-2xl">
			<h3 class="text-lg font-bold">IPFS connection</h3>
			<p class="mt-1 mb-3 text-xs text-base-content/70">
				The player joins the same private swarm as the cloud. Paste the rendezvous bootstrap
				multiaddr(s) and the swarm key (the contents of <code>swarm.key</code>).
			</p>

			<label class="form-control mb-3 w-full">
				<div class="label">
					<span class="label-text text-xs uppercase">Bootstrap multiaddrs (one per line)</span>
				</div>
				<textarea
					class="textarea-bordered textarea h-24 font-mono text-xs"
					placeholder="/ip4/127.0.0.1/tcp/14002/ws/p2p/12D3KooW..."
					bind:value={bootstrapInput}
				></textarea>
				<div class="label">
					<span class="label-text-alt text-xs text-base-content/60">
						Browser-dialable transport only — <code>/ws</code>, <code>/wss</code>, or
						<code>/webtransport</code>. Plain TCP is not supported.
					</span>
				</div>
			</label>

			<label class="form-control mb-3 w-full">
				<div class="label">
					<span class="label-text text-xs uppercase">Swarm key</span>
				</div>
				<textarea
					class="textarea-bordered textarea h-32 font-mono text-xs"
					placeholder="/key/swarm/psk/1.0.0/&#10;/base16/&#10;abcdef..."
					bind:value={swarmKeyInput}
				></textarea>
				<div class="label">
					<span class="label-text-alt text-xs text-base-content/60">
						Must match the swarm.key on the cloud / rendezvous side. Without it the
						<code>pnet</code> handshake will reject every connection.
					</span>
				</div>
			</label>

			{#if saved}
				<div class="my-2 alert alert-success"><span>Saved.</span></div>
			{/if}

			<div class="modal-action">
				<button type="button" class="btn btn-ghost btn-sm" onclick={onClose}>Close</button>
				<button type="button" class="btn btn-sm btn-primary" onclick={handleSave}>Save</button>
			</div>
		</div>
		<button type="button" class="modal-backdrop" aria-label="Close" onclick={onClose}></button>
	</div>
{/if}
