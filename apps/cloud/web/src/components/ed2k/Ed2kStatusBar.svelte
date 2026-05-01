<script lang="ts">
	import { ed2kService } from '$services/ed2k.service';

	const serviceState = ed2kService.state;

	let connecting = $state(false);

	async function handleConnect() {
		connecting = true;
		await ed2kService.connectServer();
		connecting = false;
	}
</script>

<div class="card bg-base-200">
	<div class="card-body p-4">
		<div class="flex flex-wrap items-center justify-between gap-3">
			<div class="flex flex-col gap-1">
				<div class="flex items-center gap-2">
					<span class="text-sm font-medium">Server:</span>
					{#if $serviceState.server}
						<span class="badge badge-sm badge-success">Connected</span>
						<span class="text-sm">{$serviceState.server.name}</span>
						<span class="text-xs text-base-content/60">
							({$serviceState.server.host}:{$serviceState.server.port})
						</span>
					{:else}
						<span class="badge badge-sm badge-neutral">Disconnected</span>
					{/if}
				</div>
				{#if $serviceState.server}
					<div class="text-xs text-base-content/60">
						{$serviceState.server.userCount.toLocaleString()} users · {$serviceState.server.fileCount.toLocaleString()}
						files
					</div>
					{#if $serviceState.server.message}
						<div class="text-xs text-base-content/60 italic">"{$serviceState.server.message}"</div>
					{/if}
				{/if}
			</div>

			<div class="flex items-center gap-2">
				{#if $serviceState.stats}
					<div class="text-xs text-base-content/60">
						{$serviceState.stats.activeFiles} active
					</div>
				{/if}
				<button
					class="btn btn-sm btn-primary"
					onclick={handleConnect}
					disabled={connecting || !$serviceState.initialized}
				>
					{#if connecting}
						<span class="loading loading-sm loading-spinner"></span>
					{:else if $serviceState.server}
						Reconnect
					{:else}
						Connect
					{/if}
				</button>
			</div>
		</div>
	</div>
</div>
