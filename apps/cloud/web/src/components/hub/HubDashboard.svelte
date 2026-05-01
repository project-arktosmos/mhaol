<script lang="ts">
	import classNames from 'classnames';
	import Modal from '$components/core/Modal.svelte';
	import type { HubApp } from '$types/hub.type';

	let {
		apps,
		loading = false,
		error = null,
		onstart,
		onstop,
		ondismiss
	}: {
		apps: HubApp[];
		loading?: boolean;
		error?: string | null;
		onstart: (name: string) => void;
		onstop: (name: string) => void;
		ondismiss: (name: string) => void;
	} = $props();

	let logsModalApp = $state<string | null>(null);
	let logsModalLines = $derived(
		logsModalApp ? (apps.find((a) => a.name === logsModalApp)?.logs ?? []) : []
	);

	let logEndRef: HTMLDivElement | undefined = $state();

	$effect(() => {
		if (logsModalApp && logsModalLines.length > 0 && logEndRef) {
			logEndRef.scrollIntoView({ behavior: 'smooth' });
		}
	});

	function isBusy(app: HubApp): boolean {
		return app.status === 'starting';
	}
</script>

{#if error}
	<div class="mt-4 alert alert-error">
		<span>{error}</span>
	</div>
{/if}

{#if loading && apps.length === 0}
	<div class="flex justify-center py-12">
		<span class="loading loading-lg loading-spinner"></span>
	</div>
{:else if apps.length === 0}
	<div class="mt-4 rounded-lg bg-base-200 p-8 text-center">
		<p class="opacity-50">No apps registered.</p>
	</div>
{:else}
	<div class="mt-4 flex flex-col gap-3">
		{#each apps as app (app.name)}
			<div class="card bg-base-200">
				<div class="card-body gap-3 p-4">
					<div class="flex items-center justify-between">
						<div class="flex items-center gap-2">
							{#if isBusy(app)}
								<span class="loading h-3 w-3 loading-spinner text-info"></span>
							{:else}
								<div
									class={classNames('h-2.5 w-2.5 rounded-full', {
										'bg-success': app.status === 'running',
										'bg-error': app.status === 'stopped' || app.status === 'failed',
										'bg-warning': app.status === 'unknown'
									})}
								></div>
							{/if}
							<span class="font-semibold capitalize">{app.name}</span>
						</div>
						{#if app.status === 'running'}
							<a
								href="http://localhost:{app.port}"
								target="_blank"
								rel="noopener"
								class="font-mono text-xs text-primary hover:underline"
							>
								localhost:{app.port}
							</a>
						{:else}
							<span class="font-mono text-xs text-base-content/50">:{app.port}</span>
						{/if}
					</div>

					<div class="flex items-center justify-between">
						<span
							class={classNames('badge badge-sm', {
								'badge-success': app.status === 'running',
								'badge-error': app.status === 'stopped' || app.status === 'failed',
								'badge-info': isBusy(app),
								'badge-warning': app.status === 'unknown'
							})}
						>
							{app.status}
						</span>

						<div class="flex gap-1">
							{#if app.logs.length > 0}
								<button class="btn btn-ghost btn-xs" onclick={() => (logsModalApp = app.name)}>
									Logs
								</button>
							{/if}

							{#if app.has_headless}
								{#if app.status === 'starting'}
									<button class="btn btn-disabled btn-xs" disabled>
										<span class="loading loading-xs loading-spinner"></span>
										Starting
									</button>
								{:else if app.status === 'running'}
									<button class="btn btn-xs btn-error" onclick={() => onstop(app.name)}>
										Stop
									</button>
								{:else if app.status === 'failed'}
									<button class="btn btn-ghost btn-xs" onclick={() => ondismiss(app.name)}>
										Dismiss
									</button>
									<button class="btn btn-xs btn-warning" onclick={() => onstart(app.name)}>
										Retry
									</button>
								{:else}
									<button class="btn btn-xs btn-primary" onclick={() => onstart(app.name)}>
										Start
									</button>
								{/if}
							{/if}
						</div>
					</div>
				</div>
			</div>
		{/each}
	</div>
{/if}

<Modal open={logsModalApp !== null} maxWidth="max-w-3xl" onclose={() => (logsModalApp = null)}>
	{#if logsModalApp}
		<h3 class="mb-3 text-lg font-bold capitalize">{logsModalApp} Logs</h3>
		{#if logsModalLines.length === 0}
			<p class="py-8 text-center text-sm opacity-50">No logs yet.</p>
		{:else}
			<div
				class="max-h-[70vh] overflow-y-auto rounded bg-base-300 p-3 font-mono text-xs leading-relaxed"
			>
				{#each logsModalLines as line}
					<div class="break-all whitespace-pre-wrap text-base-content/70">{line}</div>
				{/each}
				<div bind:this={logEndRef}></div>
			</div>
		{/if}
	{/if}
</Modal>
