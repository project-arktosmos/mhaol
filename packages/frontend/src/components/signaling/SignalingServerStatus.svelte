<script lang="ts">
	import classNames from 'classnames';
	import { onMount } from 'svelte';
	import { apiUrl } from '$lib/api-base';
	import type { SignalingServerStatus } from '$types/signaling.type';

	let status = $state<SignalingServerStatus | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);

	let editingPartyUrl = $state(false);
	let editValue = $state('');
	let saving = $state(false);

	let deploying = $state(false);
	let deployLogs = $state<string[]>([]);
	let deployResult = $state<{ success: boolean; code: number | null; url?: string } | null>(null);
	let deployError = $state<string | null>(null);
	let logContainer = $state<HTMLDivElement | null>(null);

	let testing = $state(false);
	let testResult = $state<boolean | null>(null);

	$effect(() => {
		if (logContainer && deployLogs.length > 0) {
			logContainer.scrollTop = logContainer.scrollHeight;
		}
	});

	onMount(async () => {
		await fetchStatus();
		await checkDeployStatus();
	});

	async function checkDeployStatus() {
		try {
			const res = await fetch(apiUrl('/api/signaling/deploy'), { method: 'HEAD' });
			if (res.status === 409) {
				deploying = true;
			}
		} catch {
			// ignore
		}
	}

	async function fetchStatus() {
		loading = true;
		error = null;
		try {
			const res = await fetch(apiUrl('/api/signaling/status'));
			if (!res.ok) throw new Error(`HTTP ${res.status}`);
			status = await res.json();
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	}

	function startEditPartyUrl() {
		if (!status) return;
		editingPartyUrl = true;
		editValue = status.partyUrl;
	}

	function cancelEdit() {
		editingPartyUrl = false;
		editValue = '';
	}

	async function savePartyUrl() {
		saving = true;
		try {
			const res = await fetch(apiUrl('/api/plugins/settings'), {
				method: 'PUT',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ plugin: 'signaling', key: 'signaling.partyUrl', value: editValue })
			});
			if (!res.ok) throw new Error(`HTTP ${res.status}`);
			editingPartyUrl = false;
			await fetchStatus();
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			saving = false;
		}
	}

	async function deploy() {
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
				deployError = body?.error ?? `Failed to start deploy: HTTP ${res.status}`;
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
						handleDeployEvent(currentEvent, currentData);
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

	function handleDeployEvent(event: string, rawData: string) {
		try {
			const data = JSON.parse(rawData);
			switch (event) {
				case 'log':
					deployLogs = [...deployLogs, data.text];
					break;
				case 'done':
					deployResult = data;
					if (data.success) {
						if (data.url) {
							saveDeployedUrl(data.url);
						}
						fetchStatus();
					}
					break;
				case 'error':
					deployError = data.message;
					break;
			}
		} catch {
			// ignore parse errors
		}
	}

	async function saveDeployedUrl(url: string) {
		try {
			const res = await fetch(apiUrl('/api/plugins/settings'), {
				method: 'PUT',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ plugin: 'signaling', key: 'signaling.partyUrl', value: url })
			});
			if (!res.ok) throw new Error(`HTTP ${res.status}`);
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		}
	}

	async function testDeployedUrl() {
		if (!status?.partyUrl) return;
		testing = true;
		testResult = null;
		try {
			const res = await fetch(apiUrl('/api/signaling/status'));
			if (!res.ok) throw new Error();
			const data = await res.json();
			testResult = data.deployedAvailable;
		} catch {
			testResult = false;
		} finally {
			testing = false;
		}
	}
</script>

<div class="flex flex-col gap-4">
	{#if error}
		<div class="alert text-sm alert-error">
			<span>{error}</span>
			<button class="btn btn-ghost btn-xs" onclick={() => (error = null)}>x</button>
		</div>
	{/if}

	{#if loading}
		<div class="flex justify-center py-4">
			<span class="loading loading-md loading-spinner"></span>
		</div>
	{:else if status}
		<div class="grid grid-cols-1 gap-4 md:grid-cols-2">
			<!-- Dev Server Card (managed by plugin connector) -->
			<div class="card bg-base-200">
				<div class="card-body gap-3 p-4">
					<div class="flex items-center justify-between">
						<h3 class="text-sm font-semibold">Local Server</h3>
						<span
							class={classNames('badge badge-sm', {
								'badge-success': status.devAvailable,
								'badge-error': !status.devAvailable
							})}
						>
							{status.devAvailable ? 'running' : 'stopped'}
						</span>
					</div>
					<span class="font-mono text-sm text-base-content/70">{status.devUrl}</span>
					<p class="text-xs text-base-content/50">Spawned automatically by the plugin connector</p>
				</div>
			</div>

			<!-- Deployed Server Card -->
			<div class="card bg-base-200">
				<div class="card-body gap-3 p-4">
					<div class="flex items-center justify-between">
						<h3 class="text-sm font-semibold">Remote Server</h3>
						{#if status.partyUrl}
							<span
								class={classNames('badge badge-sm', {
									'badge-success': status.deployedAvailable,
									'badge-error': !status.deployedAvailable
								})}
							>
								{status.deployedAvailable ? 'online' : 'offline'}
							</span>
						{:else}
							<span class="badge badge-ghost badge-sm">not deployed</span>
						{/if}
					</div>
					<div class="flex flex-col gap-1">
						<span class="text-xs text-base-content/50">Deploy name</span>
						<span class="font-mono text-sm text-base-content/70">{status.deployName}</span>
					</div>
					{#if editingPartyUrl}
						<div class="flex gap-2">
							<input
								type="text"
								class="input-bordered input input-sm flex-1 font-mono"
								placeholder="https://{status.deployName}.user.partykit.dev"
								bind:value={editValue}
								onkeydown={(e) => {
									if (e.key === 'Enter') savePartyUrl();
									if (e.key === 'Escape') cancelEdit();
								}}
							/>
							<button class="btn btn-xs btn-success" disabled={saving} onclick={savePartyUrl}>
								{#if saving}<span class="loading loading-xs loading-spinner"></span>{:else}Save{/if}
							</button>
							<button class="btn btn-ghost btn-xs" onclick={cancelEdit}>Cancel</button>
						</div>
					{:else}
						<div class="flex items-center justify-between">
							<span class="font-mono text-sm text-base-content/70">
								{status.partyUrl || 'No URL set'}
							</span>
							<div class="flex gap-1">
								{#if status.partyUrl}
									<button class="btn btn-ghost btn-xs" disabled={testing} onclick={testDeployedUrl}>
										{#if testing}
											<span class="loading loading-xs loading-spinner"></span>
										{:else}
											Test
										{/if}
									</button>
								{/if}
								<button class="btn btn-ghost btn-xs" onclick={startEditPartyUrl}>Edit</button>
							</div>
						</div>
						{#if testResult !== null}
							<span
								class={classNames('text-xs', {
									'text-success': testResult,
									'text-error': !testResult
								})}
							>
								{testResult ? 'Reachable' : 'Unreachable'}
							</span>
						{/if}
					{/if}
					<p class="text-xs text-base-content/50">
						Deploys to PartyKit (free GitHub account required for authentication)
					</p>
					<div class="flex items-center gap-2">
						<button
							class={classNames('btn btn-xs', {
								'btn-disabled': deploying,
								'btn-primary': !status.deployedAvailable,
								'btn-warning': status.deployedAvailable
							})}
							disabled={deploying}
							onclick={deploy}
						>
							{#if deploying}
								<span class="loading loading-xs loading-spinner"></span>
								Deploying...
							{:else if status.deployedAvailable}
								Redeploy
							{:else}
								Deploy
							{/if}
						</button>
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
					{#if deployLogs.length > 0 || deployError}
						<div
							bind:this={logContainer}
							class="max-h-48 overflow-y-auto rounded bg-base-300 p-2 font-mono text-xs"
						>
							{#each deployLogs as line}
								<div class="whitespace-pre-wrap">{line}</div>
							{/each}
							{#if deployError}
								<div class="text-error">{deployError}</div>
							{/if}
						</div>
					{/if}
				</div>
			</div>
		</div>

		<!-- Identity -->
		<div class="card bg-base-200">
			<div class="card-body flex-row items-center gap-3 p-4">
				<span class="text-sm font-semibold">Identity</span>
				{#if status.identityAddress}
					<code class="text-sm text-base-content/70">{status.identityAddress}</code>
				{:else}
					<span class="text-sm text-base-content/50">No identity configured</span>
				{/if}
			</div>
		</div>

		<!-- Refresh -->
		<div class="flex justify-end">
			<button class="btn btn-ghost btn-sm" onclick={fetchStatus}>Refresh Status</button>
		</div>
	{/if}
</div>
