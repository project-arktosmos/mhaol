<script lang="ts">
	import classNames from 'classnames';
	import { onMount } from 'svelte';
	import { DEFAULT_SIGNALING_URL } from 'ui-lib/lib/api-base';
	import { connectionConfigService } from 'ui-lib/services/connection-config.service';
	import { clientIdentityService } from 'ui-lib/services/client-identity.service';
	import {
		nodeConnectionService,
		type NodeConnectionPhase
	} from 'ui-lib/services/node-connection.service';
	import { generateRandomUsername } from 'ui-lib/utils/random-username';
	import { toastService } from 'ui-lib/services/toast.service';
	import { buildConnectUrl } from 'ui-lib/services/connect-url.service';
	import type { TransportMode } from 'ui-lib/types/connection-config.type';

	let {
		onconnected,
		ondisconnect
	}: {
		onconnected: () => void;
		ondisconnect?: () => void;
	} = $props();

	const defaults = connectionConfigService.defaults();
	const existingConfig = connectionConfigService.get();

	const localIdentity = clientIdentityService.loadLocal();
	let displayName = $state(localIdentity.name);
	let clientAddress = localIdentity.address;

	let transportMode = $state<TransportMode>(existingConfig?.transportMode ?? 'http');
	let serverUrl = $state(existingConfig?.serverUrl ?? defaults.serverUrl);
	let serverAddress = $state(existingConfig?.serverAddress ?? defaults.serverAddress);
	let signalingUrl = $state(existingConfig?.signalingUrl ?? defaults.signalingUrl);

	function handleNameChange(value: string) {
		displayName = value;
		clientIdentityService.updateName(value);
	}

	function randomizeName() {
		handleNameChange(generateRandomUsername());
	}

	onMount(() => {
		if (!existingConfig) {
			connectionConfigService.loadNodeDefaults().then((nodeDefaults) => {
				if (!nodeDefaults) return;
				const fresh = connectionConfigService.defaults();
				serverUrl = fresh.serverUrl;
				serverAddress = fresh.serverAddress;
				if (fresh.signalingUrl) signalingUrl = fresh.signalingUrl;
			});
		}
	});

	const connState = nodeConnectionService.state;
	let connected = $derived($connState.phase === 'ready');
	let connecting = $derived(
		$connState.phase !== 'idle' && $connState.phase !== 'ready' && $connState.phase !== 'error'
	);

	const WEBRTC_STEPS: { phase: NodeConnectionPhase; label: string }[] = [
		{ phase: 'connecting', label: 'Initialize identity' },
		{ phase: 'signaling', label: 'Connect to signaling' },
		{ phase: 'peer-discovery', label: 'Discover server peer' },
		{ phase: 'webrtc', label: 'Establish WebRTC connection' },
		{ phase: 'handshake', label: 'Exchange passports' },
		{ phase: 'ready', label: 'Ready' }
	];

	function stepStatus(
		stepPhase: NodeConnectionPhase,
		currentPhase: NodeConnectionPhase
	): 'pending' | 'active' | 'done' | 'error' {
		const stepIdx = WEBRTC_STEPS.findIndex((s) => s.phase === stepPhase);
		const currentIdx = WEBRTC_STEPS.findIndex((s) => s.phase === currentPhase);
		if (currentPhase === 'error') {
			if (stepIdx < currentIdx || currentIdx === -1) return 'done';
			return stepIdx === currentIdx ? 'error' : 'pending';
		}
		if (currentIdx === -1) return 'pending';
		if (stepIdx < currentIdx) return 'done';
		if (stepIdx === currentIdx) return 'active';
		return 'pending';
	}

	let canConnect = $derived(
		transportMode === 'http'
			? serverUrl.trim().length > 0
			: serverAddress.trim().length > 0 && signalingUrl.trim().length > 0
	);

	async function handleConnect() {
		const config = {
			transportMode,
			serverUrl: serverUrl.trim(),
			serverAddress: serverAddress.trim(),
			signalingUrl: signalingUrl.trim()
		};

		try {
			if (transportMode === 'http') {
				await nodeConnectionService.connectHttp(config);
			} else {
				await nodeConnectionService.connectWebRtc(config);
			}
			connectionConfigService.save(config);
			onconnected();
		} catch {
			// Error is already in connState
		}
	}

	function handleDisconnect() {
		nodeConnectionService.disconnect();
		connectionConfigService.clear();
		ondisconnect?.();
	}

	async function handleShare() {
		if (!existingConfig) return;
		const url = buildConnectUrl(existingConfig);
		await navigator.clipboard.writeText(url);
		toastService.success('Connection URL copied to clipboard');
	}
</script>

<div class="flex flex-col gap-4">
	<div>
		<h2 class="text-xl font-bold">Node Setup</h2>
		<p class="text-sm text-base-content/60">Configure the connection to your Mhaol node</p>
	</div>

	<!-- Connected status view -->
	{#if connected && existingConfig}
		<div class="flex items-center gap-2">
			<span class="badge gap-1 badge-success">
				<span class="h-2 w-2 rounded-full bg-success-content"></span>
				Connected
			</span>
			<span class="badge badge-outline badge-sm">
				{existingConfig.transportMode.toUpperCase()}
			</span>
		</div>

		<div class="rounded-lg bg-base-200 p-3">
			{#if existingConfig.transportMode === 'http'}
				<div class="text-sm">
					<span class="text-base-content/60">Server URL</span>
					<p class="mt-0.5 truncate font-mono">{existingConfig.serverUrl}</p>
				</div>
			{:else}
				<div class="flex flex-col gap-2 text-sm">
					<div>
						<span class="text-base-content/60">Server Address</span>
						<p class="mt-0.5 truncate font-mono">{existingConfig.serverAddress}</p>
					</div>
					<div>
						<span class="text-base-content/60">Signaling Server</span>
						<p class="mt-0.5 truncate font-mono">{existingConfig.signalingUrl}</p>
					</div>
				</div>
			{/if}
		</div>

		<div class="flex gap-2">
			<button class="btn flex-1 btn-outline btn-error" onclick={handleDisconnect}>
				Disconnect
			</button>
			<button class="btn flex-1 btn-outline btn-primary" onclick={handleShare}> Share </button>
		</div>
	{:else}
		<!-- Client identity -->
		<div class="rounded-lg bg-base-200 p-3">
			<div class="text-sm">
				<span class="text-base-content/60">Your Address</span>
				<p class="mt-0.5 truncate font-mono text-xs">{clientAddress}</p>
			</div>
		</div>

		<div class="form-control">
			<label class="label" for="display-name">
				<span class="label-text">Display Name</span>
			</label>
			<div class="flex gap-2">
				<input
					id="display-name"
					type="text"
					class="input-bordered input w-full"
					placeholder="Enter your name"
					value={displayName}
					oninput={(e) => handleNameChange(e.currentTarget.value)}
					disabled={connecting}
				/>
				<button
					class="btn btn-square self-center btn-ghost btn-sm"
					title="Generate random name"
					disabled={connecting}
					onclick={randomizeName}
				>
					&#x21bb;
				</button>
			</div>
		</div>

		<!-- Transport mode selector -->
		<div class="flex gap-2">
			<button
				class={classNames('btn flex-1 btn-sm', {
					'btn-primary': transportMode === 'http',
					'btn-ghost': transportMode !== 'http'
				})}
				disabled={connecting}
				onclick={() => (transportMode = 'http')}
			>
				HTTP
			</button>
			<button
				class={classNames('btn flex-1 btn-sm', {
					'btn-primary': transportMode === 'webrtc',
					'btn-ghost': transportMode !== 'webrtc'
				})}
				disabled={connecting}
				onclick={() => (transportMode = 'webrtc')}
			>
				WebRTC
			</button>
		</div>

		<!-- HTTP fields -->
		{#if transportMode === 'http'}
			<div class="form-control">
				<label class="label" for="server-url">
					<span class="label-text">Server URL</span>
				</label>
				<input
					id="server-url"
					type="text"
					class="input-bordered input w-full"
					placeholder="http://192.168.1.5:1530"
					bind:value={serverUrl}
					disabled={connecting}
				/>
			</div>
		{/if}

		<!-- WebRTC fields -->
		{#if transportMode === 'webrtc'}
			<div class="form-control">
				<label class="label" for="server-address">
					<span class="label-text">Server Ethereum Address</span>
				</label>
				<input
					id="server-address"
					type="text"
					class="input-bordered input w-full font-mono text-sm"
					placeholder="0x..."
					bind:value={serverAddress}
					disabled={connecting}
				/>
			</div>
			<div class="form-control">
				<label class="label" for="signaling-url">
					<span class="label-text">Signaling Server</span>
				</label>
				<input
					id="signaling-url"
					type="text"
					class="input-bordered input w-full text-sm"
					placeholder={DEFAULT_SIGNALING_URL}
					bind:value={signalingUrl}
					disabled={connecting}
				/>
			</div>
		{/if}

		<!-- WebRTC connection progress -->
		{#if transportMode === 'webrtc' && $connState.phase !== 'idle'}
			<ul class="steps steps-vertical text-sm">
				{#each WEBRTC_STEPS as step (step.phase)}
					{@const status = stepStatus(step.phase, $connState.phase)}
					<li
						class={classNames('step', {
							'step-primary': status === 'done',
							'step-info': status === 'active',
							'step-error': status === 'error'
						})}
					>
						<span class="flex items-center gap-2">
							{step.label}
							{#if status === 'active'}
								<span class="loading loading-xs loading-spinner"></span>
							{/if}
						</span>
					</li>
				{/each}
			</ul>
		{/if}

		<!-- Error display -->
		{#if $connState.error}
			<div class="alert text-sm alert-error">
				<span>{$connState.error}</span>
			</div>
		{/if}

		<!-- Connect button -->
		<button class="btn btn-primary" disabled={!canConnect || connecting} onclick={handleConnect}>
			{#if connecting}
				<span class="loading loading-sm loading-spinner"></span>
				Connecting...
			{:else}
				Connect
			{/if}
		</button>
	{/if}
</div>
