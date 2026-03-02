<script lang="ts">
	import { onMount } from 'svelte';
	import classNames from 'classnames';
	import { apiUrl } from '$lib/api-base';

	interface PluginProcess {
		id: string;
		available: boolean;
		port: number;
		url: string;
		logPrefix: string;
	}

	interface PluginSetting {
		key: string;
		value: string;
		default: string;
	}

	interface PluginCompatibility {
		mobile: boolean;
		computer: boolean;
	}

	interface Plugin {
		name: string;
		version: string;
		description: string;
		source: 'plugin' | 'addon';
		compatibility: PluginCompatibility;
		processes: PluginProcess[];
		settings: PluginSetting[];
		scheduledTasks: string[];
		schemaTables: { name: string; columns: string[] }[];
	}

	let plugins = $state<Plugin[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let expandedPlugin = $state<string | null>(null);
	let editingKey = $state<string | null>(null);
	let editValue = $state('');
	let saving = $state(false);

	onMount(async () => {
		await loadPlugins();
	});

	async function loadPlugins() {
		loading = true;
		error = null;
		try {
			const res = await fetch(apiUrl('/api/plugins'));
			if (!res.ok) throw new Error(`HTTP ${res.status}`);
			const all: Plugin[] = await res.json();
			plugins = all
				.filter((p) => p.source !== 'addon')
				.map((p) => ({
					...p,
					processes: p.processes ?? [],
					settings: p.settings ?? [],
					scheduledTasks: p.scheduledTasks ?? [],
					schemaTables: p.schemaTables ?? []
				}));
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	}

	function toggleExpand(name: string) {
		expandedPlugin = expandedPlugin === name ? null : name;
		editingKey = null;
	}

	function startEdit(setting: PluginSetting) {
		editingKey = setting.key;
		editValue = setting.value;
	}

	function cancelEdit() {
		editingKey = null;
		editValue = '';
	}

	async function saveSetting(pluginName: string, key: string) {
		saving = true;
		try {
			const res = await fetch(apiUrl('/api/plugins/settings'), {
				method: 'PUT',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ plugin: pluginName, key, value: editValue })
			});
			if (!res.ok) {
				const body = await res.json().catch(() => ({}));
				throw new Error((body as { error?: string }).error ?? `HTTP ${res.status}`);
			}
			editingKey = null;
			await loadPlugins();
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			saving = false;
		}
	}

	function formatSettingLabel(key: string): string {
		const parts = key.split('.');
		const last = parts[parts.length - 1];
		return last.replace(/([A-Z])/g, ' $1').replace(/^./, (s) => s.toUpperCase());
	}

	function isSensitive(key: string): boolean {
		return key.includes('Token') || key.includes('token') || key.includes('cookies');
	}
</script>

<div class="container mx-auto p-4">
	<div class="mb-6">
		<h1 class="text-3xl font-bold">Plugins</h1>
		<p class="text-sm opacity-70">Manage installed plugins, processes, and settings</p>
	</div>

	{#if error}
		<div class="alert alert-error mb-4">
			<span>{error}</span>
			<button class="btn btn-ghost btn-sm" onclick={() => (error = null)}>x</button>
		</div>
	{/if}

	{#if loading}
		<div class="flex justify-center py-12">
			<span class="loading loading-spinner loading-lg"></span>
		</div>
	{:else if plugins.length === 0}
		<div class="rounded-lg bg-base-200 p-8 text-center">
			<p class="opacity-50">No plugins installed.</p>
		</div>
	{:else}
		<div class="flex flex-col gap-4">
			{#each plugins as plugin (plugin.name)}
				{@const isExpanded = expandedPlugin === plugin.name}
				{@const allProcessesUp = plugin.processes.length > 0 && plugin.processes.every((p) => p.available)}
				{@const someProcessDown = plugin.processes.some((p) => !p.available)}
				{@const noProcesses = plugin.processes.length === 0}

				<div class="card bg-base-200">
					<div class="card-body p-4">
						<!-- Plugin header -->
						<button
							class="flex w-full cursor-pointer items-center justify-between text-left"
							onclick={() => toggleExpand(plugin.name)}
						>
							<div class="flex items-center gap-3">
								<div>
									<h2 class="card-title text-lg">{plugin.name}</h2>
									<p class="text-sm opacity-70">{plugin.description}</p>
								</div>
							</div>
							<div class="flex items-center gap-2">
								<span class="badge badge-ghost badge-sm">v{plugin.version}</span>
								{#if plugin.compatibility?.mobile}
									<span class="badge badge-outline badge-sm">mobile</span>
								{/if}
								{#if plugin.compatibility?.computer}
									<span class="badge badge-outline badge-sm">computer</span>
								{/if}
								{#if noProcesses}
									<span class="badge badge-info badge-sm">library</span>
								{:else if allProcessesUp}
									<span class="badge badge-success badge-sm">running</span>
								{:else if someProcessDown}
									<span class="badge badge-warning badge-sm">degraded</span>
								{:else}
									<span class="badge badge-error badge-sm">stopped</span>
								{/if}
								<svg
									class={classNames('h-4 w-4 transition-transform', {
										'rotate-180': isExpanded
									})}
									fill="none"
									stroke="currentColor"
									viewBox="0 0 24 24"
								>
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
								</svg>
							</div>
						</button>

						<!-- Expanded details -->
						{#if isExpanded}
							<div class="mt-4 flex flex-col gap-4">
								<!-- Processes -->
								{#if plugin.processes.length > 0}
									<div>
										<h3 class="mb-2 text-sm font-semibold uppercase tracking-wide opacity-60">Processes</h3>
										<div class="overflow-x-auto">
											<table class="table table-sm">
												<thead>
													<tr class="bg-base-300/50">
														<th>ID</th>
														<th>Status</th>
														<th>Port</th>
														<th>URL</th>
													</tr>
												</thead>
												<tbody>
													{#each plugin.processes as proc (proc.id)}
														<tr>
															<td class="font-mono text-sm">{proc.id}</td>
															<td>
																{#if proc.available}
																	<span class="badge badge-success badge-xs">running</span>
																{:else}
																	<span class="badge badge-error badge-xs">stopped</span>
																{/if}
															</td>
															<td class="font-mono text-sm">{proc.port}</td>
															<td class="font-mono text-sm opacity-70">{proc.url}</td>
														</tr>
													{/each}
												</tbody>
											</table>
										</div>
									</div>
								{/if}

								<!-- Settings -->
								{#if plugin.settings.length > 0}
									<div>
										<h3 class="mb-2 text-sm font-semibold uppercase tracking-wide opacity-60">Settings</h3>
										<div class="overflow-x-auto">
											<table class="table table-sm">
												<thead>
													<tr class="bg-base-300/50">
														<th>Setting</th>
														<th>Value</th>
														<th>Default</th>
														<th></th>
													</tr>
												</thead>
												<tbody>
													{#each plugin.settings as setting (setting.key)}
														<tr>
															<td>
																<span class="text-sm">{formatSettingLabel(setting.key)}</span>
																<br />
																<span class="font-mono text-xs opacity-50">{setting.key}</span>
															</td>
															<td>
																{#if editingKey === setting.key}
																	<input
																		type={isSensitive(setting.key) ? 'password' : 'text'}
																		class="input input-bordered input-sm w-full max-w-xs font-mono"
																		bind:value={editValue}
																		onkeydown={(e) => {
																			if (e.key === 'Enter') saveSetting(plugin.name, setting.key);
																			if (e.key === 'Escape') cancelEdit();
																		}}
																	/>
																{:else}
																	<span class="font-mono text-sm">
																		{#if isSensitive(setting.key) && setting.value}
																			{'*'.repeat(Math.min(setting.value.length, 16))}
																		{:else}
																			{setting.value || '-'}
																		{/if}
																	</span>
																{/if}
															</td>
															<td class="font-mono text-sm opacity-50">{setting.default || '-'}</td>
															<td>
																{#if editingKey === setting.key}
																	<div class="flex gap-1">
																		<button
																			class="btn btn-success btn-xs"
																			disabled={saving}
																			onclick={() => saveSetting(plugin.name, setting.key)}
																		>
																			{#if saving}
																				<span class="loading loading-spinner loading-xs"></span>
																			{:else}
																				Save
																			{/if}
																		</button>
																		<button class="btn btn-ghost btn-xs" onclick={cancelEdit}>
																			Cancel
																		</button>
																	</div>
																{:else}
																	<button class="btn btn-ghost btn-xs" onclick={() => startEdit(setting)}>
																		Edit
																	</button>
																{/if}
															</td>
														</tr>
													{/each}
												</tbody>
											</table>
										</div>
									</div>
								{/if}

								<!-- Capabilities summary -->
								<div>
									<h3 class="mb-2 text-sm font-semibold uppercase tracking-wide opacity-60">Capabilities</h3>
									<div class="flex flex-wrap gap-2">
										{#if plugin.schemaTables.length > 0}
											<span class="badge badge-outline badge-sm">
												{plugin.schemaTables.length} Database Table{plugin.schemaTables.length !== 1 ? 's' : ''}
											</span>
										{/if}
										{#if plugin.processes.length > 0}
											<span class="badge badge-outline badge-sm">
												{plugin.processes.length} Process{plugin.processes.length !== 1 ? 'es' : ''}
											</span>
										{/if}
										{#if plugin.settings.length > 0}
											<span class="badge badge-outline badge-sm">
												{plugin.settings.length} Setting{plugin.settings.length !== 1 ? 's' : ''}
											</span>
										{/if}
										{#if plugin.scheduledTasks.length > 0}
											<span class="badge badge-outline badge-sm">
												{plugin.scheduledTasks.length} Scheduled Task{plugin.scheduledTasks.length !== 1 ? 's' : ''}
											</span>
										{/if}
									</div>
								</div>
							</div>
						{/if}
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>
