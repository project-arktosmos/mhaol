<script lang="ts">
	import classNames from 'classnames';
	import { onMount } from 'svelte';
	import { apiUrl } from 'frontend/lib/api-base';
	import { identityService } from 'frontend/services/identity.service';
	import { identityAdapter } from 'frontend/adapters/classes/identity.adapter';
	import { playerService } from 'frontend/services/player.service';
	import { signalingAdapter } from 'frontend/adapters/classes/signaling.adapter';
	import { signalingChatService } from 'frontend/services/signaling-chat.service';
	import { sidebarService } from 'frontend/services/sidebar.service';
	import DownloadsSummary from 'frontend/components/downloads/DownloadsSummary.svelte';
	import type { SignalingServerStatus, SignalingServer } from 'frontend/types/signaling.type';
	import type { SidebarWidthMode } from 'frontend/types/sidebar.type';

	interface Props {
		classes?: string;
	}

	let { classes = '' }: Props = $props();

	const identityState = identityService.state;
	const playerState = playerService.state;
	const chatState = signalingChatService.state;
	const sidebarSettings = sidebarService.store;

	let signalingStatus = $state<SignalingServerStatus | null>(null);

	// Add server form
	let addingServer = $state(false);
	let addName = $state('');
	let addUrl = $state('');
	let addSaving = $state(false);

	// Edit server form
	let editingServerId = $state<string | null>(null);
	let editName = $state('');
	let editUrl = $state('');
	let editSaving = $state(false);

	// Deploy
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

	async function addServer() {
		if (!addName.trim() || !addUrl.trim()) return;
		addSaving = true;
		try {
			const res = await fetch(apiUrl('/api/signaling/servers'), {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ name: addName.trim(), url: addUrl.trim() })
			});
			if (!res.ok) throw new Error(`HTTP ${res.status}`);
			addingServer = false;
			addName = '';
			addUrl = '';
			await fetchSignalingStatus();
		} catch {
			// Ignore
		} finally {
			addSaving = false;
		}
	}

	function startEdit(server: SignalingServer) {
		editingServerId = server.id;
		editName = server.name;
		editUrl = server.url;
	}

	function cancelEdit() {
		editingServerId = null;
		editName = '';
		editUrl = '';
	}

	async function saveEdit() {
		if (!editingServerId || !editName.trim() || !editUrl.trim()) return;
		const server = signalingStatus?.servers.find((s) => s.id === editingServerId);
		if (!server) return;
		editSaving = true;
		try {
			const res = await fetch(apiUrl(`/api/signaling/servers/${editingServerId}`), {
				method: 'PUT',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					name: editName.trim(),
					url: editUrl.trim(),
					enabled: server.enabled
				})
			});
			if (!res.ok) throw new Error(`HTTP ${res.status}`);
			cancelEdit();
			await fetchSignalingStatus();
		} catch {
			// Ignore
		} finally {
			editSaving = false;
		}
	}

	async function toggleServer(server: SignalingServer) {
		try {
			const res = await fetch(apiUrl(`/api/signaling/servers/${server.id}`), {
				method: 'PUT',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ name: server.name, url: server.url, enabled: !server.enabled })
			});
			if (!res.ok) throw new Error(`HTTP ${res.status}`);
			await fetchSignalingStatus();
		} catch {
			// Ignore
		}
	}

	async function deleteServer(id: string) {
		try {
			const res = await fetch(apiUrl(`/api/signaling/servers/${id}`), { method: 'DELETE' });
			if (!res.ok) throw new Error(`HTTP ${res.status}`);
			await fetchSignalingStatus();
		} catch {
			// Ignore
		}
	}

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

	let localAvailable = $derived(signalingStatus?.devAvailable ?? false);
	let localUrl = $derived(
		signalingAdapter.resolveLocalUrl(signalingStatus?.devUrl ?? 'http://127.0.0.1:1999')
	);

	let servers = $derived(signalingStatus?.servers ?? []);
	let serverAvailable = $derived(localAvailable || servers.some((s) => s.enabled && s.available));

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
			<!-- LOCAL SERVER -->
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

			<!-- REMOTE SERVERS -->
			<div class="mt-3 border-t border-base-300/50 pt-2">
				<div class="flex items-center justify-between">
					<span class="text-xs font-semibold text-base-content/60">
						Remote ({servers.length})
					</span>
					<div class="flex gap-1">
						<button
							class="btn btn-ghost btn-xs"
							onclick={() => {
								addingServer = true;
								addName = '';
								addUrl = '';
							}}
						>
							Add
						</button>
						<button class="btn btn-xs btn-primary" disabled={deploying} onclick={deploySignaling}>
							{#if deploying}
								<span class="loading loading-xs loading-spinner"></span>
							{:else}
								Deploy
							{/if}
						</button>
					</div>
				</div>

				{#if addingServer}
					<div class="mt-2 flex flex-col gap-1">
						<input
							type="text"
							class="input-bordered input input-xs w-full"
							placeholder="Server name"
							bind:value={addName}
							onkeydown={(e) => {
								if (e.key === 'Enter') addServer();
								if (e.key === 'Escape') {
									addingServer = false;
								}
							}}
						/>
						<input
							type="text"
							class="input-bordered input input-xs w-full font-mono"
							placeholder="https://your-server.partykit.dev"
							bind:value={addUrl}
							onkeydown={(e) => {
								if (e.key === 'Enter') addServer();
								if (e.key === 'Escape') {
									addingServer = false;
								}
							}}
						/>
						<div class="flex gap-1">
							<button
								class="btn flex-1 btn-xs btn-success"
								disabled={addSaving}
								onclick={addServer}
							>
								{#if addSaving}<span class="loading loading-xs loading-spinner"
									></span>{:else}Save{/if}
							</button>
							<button class="btn btn-ghost btn-xs" onclick={() => (addingServer = false)}>
								Cancel
							</button>
						</div>
					</div>
				{/if}

				<div class="mt-2 flex flex-col gap-2">
					{#each servers as server (server.id)}
						{#if editingServerId === server.id}
							<div class="flex flex-col gap-1 rounded bg-base-100 p-2">
								<input
									type="text"
									class="input-bordered input input-xs w-full"
									placeholder="Server name"
									bind:value={editName}
									onkeydown={(e) => {
										if (e.key === 'Enter') saveEdit();
										if (e.key === 'Escape') cancelEdit();
									}}
								/>
								<input
									type="text"
									class="input-bordered input input-xs w-full font-mono"
									placeholder="https://..."
									bind:value={editUrl}
									onkeydown={(e) => {
										if (e.key === 'Enter') saveEdit();
										if (e.key === 'Escape') cancelEdit();
									}}
								/>
								<div class="flex gap-1">
									<button
										class="btn flex-1 btn-xs btn-success"
										disabled={editSaving}
										onclick={saveEdit}
									>
										{#if editSaving}<span class="loading loading-xs loading-spinner"
											></span>{:else}Save{/if}
									</button>
									<button class="btn btn-ghost btn-xs" onclick={cancelEdit}>Cancel</button>
								</div>
							</div>
						{:else}
							<div class="flex flex-col gap-1 rounded bg-base-100 p-2">
								<div class="flex items-center justify-between">
									<div class="flex items-center gap-2">
										<span
											class={classNames('h-1.5 w-1.5 rounded-full', {
												'bg-success': server.available && server.enabled,
												'bg-error': !server.available && server.enabled,
												'bg-base-300': !server.enabled
											})}
										></span>
										<span
											class={classNames('text-xs font-semibold', {
												'text-base-content/40': !server.enabled
											})}
										>
											{server.name}
										</span>
									</div>
									<div class="flex gap-0.5">
										<button class="btn btn-ghost btn-xs" onclick={() => toggleServer(server)}>
											{server.enabled ? 'Off' : 'On'}
										</button>
										<button class="btn btn-ghost btn-xs" onclick={() => startEdit(server)}>
											Edit
										</button>
										<button
											class="btn text-error btn-ghost btn-xs"
											onclick={() => deleteServer(server.id)}
										>
											Del
										</button>
									</div>
								</div>
								<span
									class={classNames('truncate font-mono text-xs', {
										'text-base-content/60': server.enabled,
										'text-base-content/30': !server.enabled
									})}
									title={server.url}
								>
									{server.url}
								</span>
							</div>
						{/if}
					{/each}

					{#if servers.length === 0 && !addingServer}
						<span class="text-xs text-base-content/40">No remote servers</span>
					{/if}
				</div>

				{#if deployLogs.length > 0 || deployError}
					<div class="mt-2 max-h-32 overflow-y-auto rounded bg-base-300 p-2 font-mono text-xs">
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
						class={classNames('mt-1 badge badge-sm', {
							'badge-success': deployResult.success,
							'badge-error': !deployResult.success
						})}
					>
						{deployResult.success ? 'Deployed' : `Failed (exit ${deployResult.code})`}
					</span>
				{/if}
			</div>
		{/if}
	</div>

	<div class="mt-4 border-t border-base-300 pt-4">
		<DownloadsSummary />
	</div>
</aside>
