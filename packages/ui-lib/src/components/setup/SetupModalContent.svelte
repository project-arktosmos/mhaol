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
	import { buildInvite, parseInvite } from 'ui-lib/services/connect-invite.service';
	import { blo } from 'blo';
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

	type ConnectionTab = 'invite' | TransportMode;
	let activeTab = $state<ConnectionTab>('invite');
	let transportMode = $state<TransportMode>(existingConfig?.transportMode ?? 'http');
	let serverUrl = $state(existingConfig?.serverUrl ?? defaults.serverUrl);
	let serverAddress = $state(existingConfig?.serverAddress ?? defaults.serverAddress);
	let signalingUrl = $state(existingConfig?.signalingUrl ?? defaults.signalingUrl);

	let inviteInput = $state('');

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
		transportMode === 'http' || transportMode === 'ws'
			? serverUrl.trim().length > 0
			: serverAddress.trim().length > 0 && signalingUrl.trim().length > 0
	);

	let parsedInvite = $derived(inviteInput.trim() ? parseInvite(inviteInput.trim()) : null);
	let canConnectInvite = $derived(parsedInvite !== null);

	async function handleConnectInvite() {
		if (!parsedInvite) return;

		try {
			if (parsedInvite.transportMode === 'http') {
				await nodeConnectionService.connectHttp(parsedInvite);
			} else if (parsedInvite.transportMode === 'ws') {
				await nodeConnectionService.connectWs(parsedInvite);
			} else {
				await nodeConnectionService.connectWebRtc(parsedInvite);
			}
			connectionConfigService.save(parsedInvite);
			onconnected();
		} catch {
			// Error is already in connState
		}
	}

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
			} else if (transportMode === 'ws') {
				await nodeConnectionService.connectWs(config);
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

	async function handleCopyInvite() {
		if (!existingConfig) return;
		const json = buildInvite(existingConfig);
		await navigator.clipboard.writeText(json);
		toastService.success('Invite copied to clipboard');
	}
</script>

<div class="flex flex-col gap-4">
	<div>
		<h2 class="text-xl font-bold">Node Setup</h2>
		<p class="text-sm text-base-content/60">Configure the connection to your Mhaol node</p>
	</div>

	<!-- Connected status view -->
	{#if connected && existingConfig}
		<div class="flex items-center gap-3 rounded-lg bg-base-200 p-3">
			{#if clientAddress}
				<img
					src={blo(clientAddress as `0x${string}`)}
					alt="identicon"
					class="h-10 w-10 rounded-full"
				/>
			{/if}
			<div class="min-w-0">
				<div class="flex items-center gap-2">
					<span class="badge gap-1 badge-sm badge-success">
						<span class="h-1.5 w-1.5 rounded-full bg-success-content"></span>
						Connected
					</span>
					<span class="badge badge-outline badge-sm">
						{existingConfig.transportMode.toUpperCase()}
					</span>
				</div>
				<p class="mt-1 truncate font-mono text-xs">{clientAddress}</p>
			</div>
		</div>

		<div class="rounded-lg bg-base-200 p-3">
			{#if existingConfig.transportMode === 'http' || existingConfig.transportMode === 'ws'}
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

		<div class="form-control">
			<label class="label" for="invite-output">
				<span class="label-text text-base-content/60">Invite</span>
			</label>
			<textarea
				id="invite-output"
				class="textarea-bordered textarea w-full font-mono text-xs"
				readonly
				rows="2"
				value={buildInvite(existingConfig)}
			></textarea>
		</div>

		<div class="flex gap-2">
			<button class="btn flex-1 btn-outline btn-error" onclick={handleDisconnect}>
				Disconnect
			</button>
			<button class="btn flex-1 btn-outline btn-primary" onclick={handleCopyInvite}>
				Copy Invite
			</button>
		</div>
	{:else}
		<!-- Client identity -->
		<div class="flex items-center gap-3 rounded-lg bg-base-200 p-3">
			{#if clientAddress}
				<img
					src={blo(clientAddress as `0x${string}`)}
					alt="identicon"
					class="h-10 w-10 rounded-full"
				/>
			{/if}
			<div class="min-w-0 text-sm">
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

		<!-- Connection mode tabs -->
		<div class="flex flex-wrap gap-1">
			{#each [{ id: 'invite', label: 'Paste Invite' }, { id: 'http', label: 'HTTP' }, { id: 'ws', label: 'WebSocket' }, { id: 'webrtc', label: 'WebRTC' }] as tab (tab.id)}
				<button
					class={classNames('btn btn-sm', {
						'btn-primary': activeTab === tab.id,
						'btn-ghost': activeTab !== tab.id
					})}
					disabled={connecting}
					onclick={() => {
						activeTab = tab.id as ConnectionTab;
						if (tab.id !== 'invite') transportMode = tab.id as TransportMode;
					}}
				>
					{tab.label}
				</button>
			{/each}
		</div>

		<!-- Invite paste tab -->
		{#if activeTab === 'invite'}
			<div class="form-control">
				<textarea
					id="invite-input"
					class={classNames('textarea-bordered textarea w-full font-mono text-xs', {
						'textarea-error': inviteInput.trim() && !parsedInvite
					})}
					placeholder={'{"transport":"ws","serverUrl":"http://192.168.1.5:1530"}'}
					rows="3"
					bind:value={inviteInput}
					disabled={connecting}
				></textarea>
				{#if inviteInput.trim() && !parsedInvite}
					<label class="label">
						<span class="label-text-alt text-error">Invalid invite JSON</span>
					</label>
				{/if}
			</div>
		{/if}

		<!-- HTTP / WS fields -->
		{#if activeTab === 'http' || activeTab === 'ws'}
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
		{#if activeTab === 'webrtc'}
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
		{#if activeTab === 'webrtc' && $connState.phase !== 'idle'}
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
		{#if activeTab === 'invite'}
			<button
				class="btn btn-primary"
				disabled={!canConnectInvite || connecting}
				onclick={handleConnectInvite}
			>
				{#if connecting}
					<span class="loading loading-sm loading-spinner"></span>
					Connecting...
				{:else}
					Connect
				{/if}
			</button>
		{:else}
			<button class="btn btn-primary" disabled={!canConnect || connecting} onclick={handleConnect}>
				{#if connecting}
					<span class="loading loading-sm loading-spinner"></span>
					Connecting...
				{:else}
					Connect
				{/if}
			</button>
		{/if}
	{/if}
</div>
