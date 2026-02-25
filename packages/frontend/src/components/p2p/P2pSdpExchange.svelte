<script lang="ts">
	import classNames from 'classnames';
	import { p2pService } from '$services/p2p.service';
	import { p2pAdapter } from '$adapters/classes/p2p.adapter';
	import P2pQrDisplay from './P2pQrDisplay.svelte';
	import P2pQrScanner from './P2pQrScanner.svelte';

	const state = p2pService.state;

	let remoteSdpInput = '';
	let showQrScanner = false;
	let copied = false;

	$: showRoleSelector = $state.phase === 'idle';
	$: showLocalSdp = $state.localSdpEncoded !== null;
	$: showRemoteInput = $state.phase === 'waiting-answer' || $state.phase === 'idle';
	$: isInitiator = $state.role === 'initiator';

	async function handleCreateOffer() {
		await p2pService.createOffer();
	}

	async function handlePasteRemote() {
		if (!remoteSdpInput.trim()) return;

		if ($state.phase === 'waiting-answer') {
			await p2pService.acceptAnswer(remoteSdpInput.trim());
		} else {
			await p2pService.acceptOffer(remoteSdpInput.trim());
		}
		remoteSdpInput = '';
	}

	async function handleQrScan(event: CustomEvent<{ data: string }>) {
		showQrScanner = false;

		if ($state.phase === 'waiting-answer') {
			await p2pService.acceptAnswer(event.detail.data);
		} else {
			await p2pService.acceptOffer(event.detail.data);
		}
	}

	async function copyLocalSdp() {
		if (!$state.localSdpEncoded) return;
		try {
			if (navigator.clipboard?.writeText) {
				await navigator.clipboard.writeText($state.localSdpEncoded);
			} else {
				const textarea = document.createElement('textarea');
				textarea.value = $state.localSdpEncoded;
				textarea.style.position = 'fixed';
				textarea.style.opacity = '0';
				document.body.appendChild(textarea);
				textarea.select();
				document.execCommand('copy');
				document.body.removeChild(textarea);
			}
			copied = true;
			setTimeout(() => (copied = false), 1500);
		} catch {
			// Copy failed silently — user can manually select from textarea
		}
	}

	function handleReset() {
		p2pService.reset();
		remoteSdpInput = '';
		showQrScanner = false;
		copied = false;
	}
</script>

<div class="card bg-base-200">
	<div class="card-body gap-4">
		<div class="flex items-center justify-between">
			<h2 class="card-title text-base">Connection</h2>
			<span class={classNames('badge badge-sm', p2pAdapter.phaseBadgeClass($state.phase))}>
				{p2pAdapter.phaseLabel($state.phase)}
			</span>
		</div>

		{#if $state.error}
			<div class="alert alert-error text-sm">{$state.error}</div>
		{/if}

		<!-- Role Selection -->
		{#if showRoleSelector}
			<div class="flex flex-col gap-3">
				<p class="text-sm text-base-content/60">Choose your role to start a connection:</p>
				<button class="btn btn-primary btn-sm" on:click={handleCreateOffer}>
					Create Invite (Initiator)
				</button>
				<div class="divider text-xs text-base-content/40">OR</div>
				<p class="text-sm text-base-content/60">Paste or scan an invite from someone else:</p>
			</div>
		{/if}

		<!-- Local SDP Display -->
		{#if showLocalSdp}
			<div class="flex flex-col gap-3">
				<div class="flex items-center justify-between">
					<span class="text-sm font-medium">
						{isInitiator ? 'Your Invite (share this)' : 'Your Answer (share this)'}
					</span>
					<button class="btn btn-ghost btn-xs" on:click={copyLocalSdp}>
						{copied ? 'Copied!' : 'Copy'}
					</button>
				</div>
				<textarea
					class="textarea textarea-bordered h-24 font-mono text-xs"
					readonly
					value={$state.localSdpEncoded}
				></textarea>
				<P2pQrDisplay data={$state.localSdpEncoded} />
			</div>
		{/if}

		<!-- Remote SDP Input -->
		{#if showRemoteInput}
			<div class="flex flex-col gap-3">
				<span class="text-sm font-medium">
					{$state.phase === 'waiting-answer'
						? 'Paste the answer from the other peer:'
						: 'Paste the invite from the other peer:'}
				</span>
				<textarea
					class="textarea textarea-bordered h-24 font-mono text-xs"
					placeholder="Paste encoded SDP here..."
					bind:value={remoteSdpInput}
				></textarea>
				<div class="flex gap-2">
					<button
						class="btn btn-primary btn-sm flex-1"
						on:click={handlePasteRemote}
						disabled={!remoteSdpInput.trim()}
					>
						Apply
					</button>
					<button
						class="btn btn-secondary btn-sm"
						on:click={() => (showQrScanner = !showQrScanner)}
					>
						{showQrScanner ? 'Hide Scanner' : 'Scan QR'}
					</button>
				</div>
				{#if showQrScanner}
					<P2pQrScanner on:scan={handleQrScan} on:cancel={() => (showQrScanner = false)} />
				{/if}
			</div>
		{/if}

		<!-- Reset -->
		{#if $state.phase !== 'idle'}
			<button class="btn btn-ghost btn-sm" on:click={handleReset}>Reset Connection</button>
		{/if}
	</div>
</div>
