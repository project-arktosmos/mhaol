<script lang="ts">
	import classNames from 'classnames';
	import { torrentService } from '$services/torrent.service';

	const state = torrentService.state;

	// Download path editing
	let downloadPathInput = '';
	let editingPath = false;

	// Debug info
	let showDebug = false;
	let debugLogs: string[] = [];
	let loadingDebug = false;

	// Clear storage confirmation
	let confirmClear = false;

	$: if ($state.downloadPath && !editingPath) {
		downloadPathInput = $state.downloadPath;
	}

	async function handleSetDownloadPath() {
		if (downloadPathInput.trim()) {
			await torrentService.setDownloadPath(downloadPathInput.trim());
			editingPath = false;
		}
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

<div class="card bg-base-200">
	<div class="card-body gap-4">
		<h2 class="card-title text-lg">Settings</h2>

		<!-- Connection Status -->
		<div
			class={classNames('rounded-lg p-3', {
				'bg-success/10': $state.initialized,
				'bg-warning/10': !$state.initialized
			})}
		>
			<div class="flex items-center gap-2">
				<div
					class={classNames('h-2 w-2 rounded-full', {
						'bg-success': $state.initialized,
						'bg-warning': !$state.initialized
					})}
				></div>
				<span class="text-sm font-medium">
					{#if $state.initialized}
						Server Connected
					{:else}
						Server Disconnected
					{/if}
				</span>
			</div>
		</div>

		<!-- Download Path -->
		<div class="form-control">
			<label class="label" for="download-path">
				<span class="label-text">Download Path</span>
			</label>
			<div class="flex items-center gap-2">
				<input
					id="download-path"
					type="text"
					class="input input-bordered flex-1"
					bind:value={downloadPathInput}
					on:focus={() => (editingPath = true)}
					placeholder="/path/to/downloads"
					title={$state.downloadPath}
				/>
				<button class="btn btn-outline btn-sm" on:click={handleSetDownloadPath}>
					Set
				</button>
			</div>
		</div>

		<!-- Clear Storage -->
		<div class="form-control">
			<button
				class={classNames('btn btn-sm', {
					'btn-error': confirmClear,
					'btn-outline': !confirmClear
				})}
				on:click={handleClearStorage}
			>
				{#if confirmClear}
					Confirm Clear Storage
				{:else}
					Clear Storage
				{/if}
			</button>
			{#if confirmClear}
				<span class="label">
					<span class="label-text-alt text-warning"
						>This will delete all downloaded files and persistence data</span
					>
				</span>
			{/if}
		</div>

		<!-- Debug Info (Collapsible) -->
		<div class="divider my-1"></div>
		<button
			class="flex w-full items-center justify-between text-sm text-base-content/70 hover:text-base-content"
			on:click={() => {
				showDebug = !showDebug;
				if (showDebug && debugLogs.length === 0) {
					handleFetchDebug();
				}
			}}
		>
			<span>Debug Info</span>
			<svg
				xmlns="http://www.w3.org/2000/svg"
				class="h-4 w-4 transition-transform"
				class:rotate-180={showDebug}
				fill="none"
				viewBox="0 0 24 24"
				stroke="currentColor"
			>
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="2"
					d="M19 9l-7 7-7-7"
				/>
			</svg>
		</button>

		{#if showDebug}
			<div class="mt-2 flex flex-col gap-2">
				<button
					class="btn btn-ghost btn-xs self-end"
					on:click={handleFetchDebug}
					disabled={loadingDebug}
				>
					{#if loadingDebug}
						<span class="loading loading-spinner loading-xs"></span>
					{:else}
						Refresh
					{/if}
				</button>
				<div
					class="max-h-64 overflow-auto rounded-lg bg-base-300 p-3 font-mono text-xs"
				>
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
</div>
