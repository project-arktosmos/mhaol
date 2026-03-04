<script lang="ts">
	import classNames from 'classnames';
	import { onMount } from 'svelte';
	import { apiUrl } from '$lib/api-base';
	import { identityService } from '$services/identity.service';
	import { identityAdapter } from '$adapters/classes/identity.adapter';
	import { playerService } from '$services/player.service';
	import { signalingAdapter } from '$adapters/classes/signaling.adapter';
	import { signalingChatService } from '$services/signaling-chat.service';
	import { sidebarService } from '$services/sidebar.service';
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

	let signalingStatus = $state<SignalingServerStatus | null>(null);

	let editingPartyUrl = $state(false);
	let editValue = $state('');
	let savingUrl = $state(false);
	let deploying = $state(false);
	let deployLogs = $state<string[]>([]);
	let deployError = $state<string | null>(null);
	let deployResult = $state<{ success: boolean; code: number | null; url?: string } | null>(null);

	async function fetchSignalingStatus() {
		try {
			const res = await fetch(apiUrl('/api/signaling/status'));
			if (res.ok) signalingStatus = await res.json();
		} catch {
			// Ignore
		}
	}

	onMount(() => {
		fetchSignalingStatus();
	});

	async function deploySignaling() {
		deploying = true;
		deployLogs = [];
		deployResult = null;
		deployError = null;

		try {
			const res = await fetch(apiUrl('/api/signaling/deploy'));

			if (res.status === 409) {
				deployError = 'A deploy is already in progress';
				deploying = false;
				return;
			}

			if (!res.ok || !res.body) {
				const body = await res.json().catch(() => null);
				deployError = body?.error ?? `Deploy failed: HTTP ${res.status}`;
				deploying = false;
				return;
			}

			const reader = res.body.getReader();
			const decoder = new TextDecoder();
			let buffer = '';

			while (true) {
				const { done, value } = await reader.read();
				if (done) break;

				buffer += decoder.decode(value, { stream: true });
				const lines = buffer.split('\n');
				buffer = lines.pop() ?? '';

				let currentEvent = '';
				let currentData = '';

				for (const line of lines) {
					if (line.startsWith('event:')) {
						currentEvent = line.slice(6).trim();
					} else if (line.startsWith('data:')) {
						currentData = line.slice(5).trim();
					} else if (line === '' && currentEvent && currentData) {
						try {
							const data = JSON.parse(currentData);
							if (currentEvent === 'log') {
								deployLogs = [...deployLogs, data.text];
							} else if (currentEvent === 'done') {
								deployResult = data;
								if (data.success) {
									await fetchSignalingStatus();
								}
							} else if (currentEvent === 'error') {
								deployError = data.message;
							}
						} catch {
							// ignore parse errors
						}
						currentEvent = '';
						currentData = '';
					}
				}
			}
		} catch (e) {
			deployError = e instanceof Error ? e.message : 'Deploy failed';
		} finally {
			deploying = false;
		}
	}

	async function savePartyUrl() {
		savingUrl = true;
		try {
			const res = await fetch(apiUrl('/api/plugins/settings'), {
				method: 'PUT',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ plugin: 'signaling', key: 'signaling.partyUrl', value: editValue })
			});
			if (!res.ok) throw new Error(`HTTP ${res.status}`);
			editingPartyUrl = false;
			await fetchSignalingStatus();
		} catch {
			// Ignore
		} finally {
			savingUrl = false;
		}
	}

	let localAvailable = $derived(signalingStatus?.devAvailable ?? false);
	let localUrl = $derived(
		signalingAdapter.resolveLocalUrl(signalingStatus?.devUrl ?? 'http://127.0.0.1:1999')
	);

	let remoteAvailable = $derived(signalingStatus?.deployedAvailable ?? false);
	let remoteUrl = $derived(signalingStatus?.partyUrl ?? '');

	let serverAvailable = $derived(localAvailable || remoteAvailable);

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
					class={classNames('btn join-item btn-xs', {
						'btn-active': $sidebarSettings.widthMode === mode
					})}
					onclick={() => sidebarService.setWidthMode(mode)}
				>
					{label}
				</button>
			{/each}
		</div>
	</div>

	<h2 class="mb-3 text-sm font-semibold tracking-wide text-base-content/50 uppercase">
		Identities
	</h2>

	{#if $identityState.loading}
		<div class="flex justify-center py-4">
			<span class="loading loading-sm loading-spinner"></span>
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
			<h2 class="text-sm font-semibold tracking-wide text-base-content/50 uppercase">
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
			<!-- LOCAL SERVER (always visible, primary) -->
			<div class="flex flex-col gap-2">
				<div class="flex items-center justify-between">
					<span class="text-xs font-semibold text-base-content/60">Local</span>
					<span
						class={classNames('badge badge-xs', {
							'badge-success': localAvailable,
							'badge-error': !localAvailable
						})}
					>
						{localAvailable ? 'running' : 'stopped'}
					</span>
				</div>
				<span class="truncate font-mono text-xs text-base-content/60" title={localUrl}>
					{localUrl}
				</span>

				{#if localAvailable}
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
									<span class="badge badge-outline font-mono badge-sm">
										{signalingAdapter.shortAddress($playerState.remotePeerId)}
									</span>
								</div>
							{/if}
						</div>
					{/if}

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
										<span class="badge badge-outline font-mono badge-sm">
											{signalingAdapter.shortAddress(peerId)}
										</span>
									{/each}
								</div>
							{:else}
								<span class="text-xs opacity-40">No peers</span>
							{/if}
						</div>
					{/if}
				{/if}
			</div>

			<!-- REMOTE SERVER (collapsible, secondary) -->
			<div class="mt-3 border-t border-base-300/50 pt-2">
				<span class="flex items-center gap-2 text-xs font-semibold text-base-content/60">
					Remote
					{#if remoteUrl}
						<span
							class={classNames('h-1.5 w-1.5 rounded-full', {
								'bg-success': remoteAvailable,
								'bg-error': !remoteAvailable
							})}
						></span>
					{/if}
				</span>

				<div class="mt-2 flex flex-col gap-2">
					{#if remoteUrl}
						<span class="truncate font-mono text-xs text-base-content/60" title={remoteUrl}>
							{remoteUrl}
						</span>
					{:else}
						<span class="text-xs text-base-content/40">Not configured</span>
					{/if}

					{#if editingPartyUrl}
						<div class="flex flex-col gap-1">
							<input
								type="text"
								class="input-bordered input input-xs w-full font-mono"
								placeholder="https://your-server.partykit.dev"
								bind:value={editValue}
								onkeydown={(e) => {
									if (e.key === 'Enter') savePartyUrl();
									if (e.key === 'Escape') {
										editingPartyUrl = false;
										editValue = '';
									}
								}}
							/>
							<div class="flex gap-1">
								<button
									class="btn flex-1 btn-xs btn-success"
									disabled={savingUrl}
									onclick={savePartyUrl}
								>
									{#if savingUrl}<span class="loading loading-xs loading-spinner"
										></span>{:else}Save{/if}
								</button>
								<button
									class="btn btn-ghost btn-xs"
									onclick={() => {
										editingPartyUrl = false;
										editValue = '';
									}}
								>
									Cancel
								</button>
							</div>
						</div>
					{:else}
						<div class="flex gap-1">
							<button
								class="btn flex-1 btn-xs btn-primary"
								disabled={deploying}
								onclick={deploySignaling}
							>
								{#if deploying}
									<span class="loading loading-xs loading-spinner"></span>
									Deploying...
								{:else}
									Deploy
								{/if}
							</button>
							<button
								class="btn btn-ghost btn-xs"
								onclick={() => {
									editingPartyUrl = true;
									editValue = signalingStatus?.partyUrl ?? '';
								}}
							>
								Set URL
							</button>
						</div>
					{/if}

					{#if deployLogs.length > 0 || deployError}
						<div class="max-h-32 overflow-y-auto rounded bg-base-300 p-2 font-mono text-xs">
							{#each deployLogs as line}
								<div class="whitespace-pre-wrap">{line}</div>
							{/each}
							{#if deployError}
								<div class="text-error">{deployError}</div>
							{/if}
						</div>
					{/if}
					{#if deployResult}
						<span
							class={classNames('badge badge-sm', {
								'badge-success': deployResult.success,
								'badge-error': !deployResult.success
							})}
						>
							{deployResult.success ? 'Deployed' : `Failed (exit ${deployResult.code})`}
						</span>
					{/if}
				</div>
			</div>
		{/if}
	</div>

	<div class="mt-4 border-t border-base-300 pt-4">
		<DownloadsSummary />
	</div>
</aside>
