<script lang="ts">
	import classNames from 'classnames';
	import { torrentService } from '$services/torrent.service';
	import ConnectionStatus from '$components/core/ConnectionStatus.svelte';

	const torrentState = torrentService.state;

	let editingPath = $state(false);
	let pathInput = $state('');

	// Debug info
	let showDebug = $state(false);
	let debugLogs = $state<string[]>([]);
	let loadingDebug = $state(false);

	// Clear storage confirmation
	let confirmClear = $state(false);

	function startEditing() {
		pathInput = $torrentState.downloadPath || '';
		editingPath = true;
	}

	async function savePath() {
		if (pathInput.trim()) {
			await torrentService.setDownloadPath(pathInput.trim());
		}
		editingPath = false;
	}

	function cancelEditing() {
		editingPath = false;
	}

	async function handleFetchDebug() {
		loadingDebug = true;
		debugLogs = await torrentService.getDebugInfo();
		loadingDebug = false;
	}

	async function handleClearStorage() {
		if (!confirmClear) {
			confirmClear = true;
			return;
		}
		await torrentService.clearStorage();
		confirmClear = false;
	}
</script>

<div class="pr-8">
	<h3 class="text-lg font-bold">Downloads</h3>
	<p class="text-sm text-base-content/60">Configure download path and storage</p>
</div>

<div class="mt-4 flex flex-col gap-4">
	<!-- Connection Status -->
	<ConnectionStatus connected={$torrentState.initialized} />

	<!-- Download Path -->
	<div class="form-control">
		<label class="label" for="download-path">
			<span class="label-text">Download Path</span>
		</label>

		{#if editingPath}
			<div class="flex gap-2">
				<input
					id="download-path"
					type="text"
					class="input-bordered input flex-1"
					bind:value={pathInput}
					onkeydown={(e) => e.key === 'Enter' && savePath()}
				/>
				<button class="btn btn-sm btn-primary" onclick={savePath}>Save</button>
				<button class="btn btn-ghost btn-sm" onclick={cancelEditing}>Cancel</button>
			</div>
		{:else}
			<div class="flex items-center gap-2">
				<code class="flex-1 truncate rounded bg-base-300 px-3 py-2 text-sm">
					{$torrentState.downloadPath || 'Not configured'}
				</code>
				<button class="btn btn-ghost btn-sm" onclick={startEditing} title="Change download path">
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="h-4 w-4"
						fill="none"
						viewBox="0 0 24 24"
						stroke="currentColor"
						stroke-width="2"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z"
						/>
					</svg>
				</button>
			</div>
		{/if}
	</div>

	<!-- Clear Storage -->
	<div class="form-control">
		<button
			class={classNames('btn btn-sm', {
				'btn-error': confirmClear,
				'btn-outline': !confirmClear
			})}
			onclick={handleClearStorage}
		>
			{#if confirmClear}
				Confirm Clear Storage
			{:else}
				Clear Storage
			{/if}
		</button>
		{#if confirmClear}
			<span class="label">
				<span class="label-text-alt text-warning">
					This will delete all downloaded files and persistence data
				</span>
			</span>
		{/if}
	</div>

	<!-- Debug Info -->
	<div class="divider my-1"></div>
	<button
		class="flex w-full items-center justify-between text-sm text-base-content/70 hover:text-base-content"
		onclick={() => {
			showDebug = !showDebug;
			if (showDebug && debugLogs.length === 0) handleFetchDebug();
		}}
	>
		<span>Debug Info</span>
		<svg
			xmlns="http://www.w3.org/2000/svg"
			class={classNames('h-4 w-4 transition-transform', { 'rotate-180': showDebug })}
			fill="none"
			viewBox="0 0 24 24"
			stroke="currentColor"
		>
			<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
		</svg>
	</button>

	{#if showDebug}
		<div class="flex flex-col gap-2">
			<button
				class="btn self-end btn-ghost btn-xs"
				onclick={handleFetchDebug}
				disabled={loadingDebug}
			>
				{#if loadingDebug}
					<span class="loading loading-xs loading-spinner"></span>
				{:else}
					Refresh
				{/if}
			</button>
			<div class="max-h-64 overflow-auto rounded-lg bg-base-300 p-3 font-mono text-xs">
				{#if debugLogs.length === 0}
					<p class="text-base-content/50">No debug info available</p>
				{:else}
					{#each debugLogs as line}
						<p class="whitespace-pre-wrap">{line}</p>
					{/each}
				{/if}
			</div>
		</div>
	{/if}
</div>
