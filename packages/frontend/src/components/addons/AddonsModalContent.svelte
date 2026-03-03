<script lang="ts">
	import { onMount } from 'svelte';
	import { apiUrl } from '$lib/api-base';

	interface AddonSetting {
		key: string;
		value: string;
		default: string;
	}

	interface AddonLinkSource {
		service: string;
		label: string;
		mediaTypeId: string;
		categoryId?: string | null;
	}

	interface Addon {
		name: string;
		version: string;
		description: string;
		settings: AddonSetting[];
		scheduledTasks: string[];
		schemaTables: { name: string; columns: string[] }[];
		linkSources: AddonLinkSource[];
	}

	let addons = $state<Addon[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let editingKey = $state<string | null>(null);
	let editValue = $state('');
	let saving = $state(false);

	onMount(() => {
		loadAddons();
	});

	async function loadAddons() {
		loading = true;
		error = null;
		try {
			const res = await fetch(apiUrl('/api/addons'));
			if (!res.ok) throw new Error(`HTTP ${res.status}`);
			addons = await res.json();
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	}

	function startEdit(setting: AddonSetting) {
		editingKey = setting.key;
		editValue = setting.value;
	}

	function cancelEdit() {
		editingKey = null;
		editValue = '';
	}

	async function saveSetting(addonName: string, key: string) {
		saving = true;
		try {
			const res = await fetch(apiUrl('/api/addons/settings'), {
				method: 'PUT',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ addon: addonName, key, value: editValue })
			});
			if (!res.ok) {
				const body = await res.json().catch(() => ({}));
				throw new Error((body as { error?: string }).error ?? `HTTP ${res.status}`);
			}
			editingKey = null;
			await loadAddons();
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
		return (
			key.includes('Key') ||
			key.includes('key') ||
			key.includes('Token') ||
			key.includes('token') ||
			key.includes('secret') ||
			key.includes('Secret')
		);
	}
</script>

<div class="pr-8">
	<h3 class="text-lg font-bold">Addons</h3>
	<p class="text-sm text-base-content/60">Manage installed addons and their settings</p>
</div>

{#if error}
	<div class="mt-4 alert alert-error">
		<span>{error}</span>
		<button class="btn btn-ghost btn-sm" onclick={() => (error = null)}>x</button>
	</div>
{/if}

{#if loading}
	<div class="flex justify-center py-12">
		<span class="loading loading-lg loading-spinner"></span>
	</div>
{:else if addons.length === 0}
	<div class="mt-4 rounded-lg bg-base-200 p-8 text-center">
		<p class="opacity-50">No addons installed.</p>
	</div>
{:else}
	<div class="mt-4 flex flex-col gap-4">
		{#each addons as addon (addon.name)}
			<div class="card bg-base-200">
				<div class="card-body p-4">
					<!-- Header -->
					<div class="flex items-center justify-between">
						<div>
							<h2 class="card-title text-lg">{addon.name}</h2>
							<p class="text-sm opacity-70">{addon.description}</p>
						</div>
						<div class="flex items-center gap-2">
							<span class="badge badge-ghost badge-sm">v{addon.version}</span>
						</div>
					</div>

					<!-- Link sources -->
					{#if addon.linkSources.length > 0}
						<div class="mt-3">
							<h3 class="mb-2 text-sm font-semibold tracking-wide uppercase opacity-60">
								Link Sources
							</h3>
							<div class="flex flex-wrap gap-2">
								{#each addon.linkSources as ls}
									<span class="badge badge-outline badge-sm">
										{ls.label} &middot; {ls.mediaTypeId}{#if ls.categoryId}/{ls.categoryId}{/if}
									</span>
								{/each}
							</div>
						</div>
					{/if}

					<!-- Database tables -->
					{#if addon.schemaTables.length > 0}
						<div class="mt-3">
							<h3 class="mb-2 text-sm font-semibold tracking-wide uppercase opacity-60">
								Database Tables
							</h3>
							<div class="flex flex-col gap-2">
								{#each addon.schemaTables as table}
									<div>
										<span class="font-mono text-sm font-semibold">{table.name}</span>
										<span class="ml-2 text-xs opacity-50">
											{table.columns.join(', ')}
										</span>
									</div>
								{/each}
							</div>
						</div>
					{/if}

					<!-- Scheduled tasks -->
					{#if addon.scheduledTasks.length > 0}
						<div class="mt-3">
							<h3 class="mb-2 text-sm font-semibold tracking-wide uppercase opacity-60">
								Scheduled Tasks
							</h3>
							<div class="flex flex-wrap gap-2">
								{#each addon.scheduledTasks as task}
									<span class="badge badge-outline font-mono badge-sm">{task}</span>
								{/each}
							</div>
						</div>
					{/if}

					<!-- Settings -->
					{#if addon.settings.length > 0}
						<div class="mt-3">
							<h3 class="mb-2 text-sm font-semibold tracking-wide uppercase opacity-60">
								Settings
							</h3>
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
										{#each addon.settings as setting (setting.key)}
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
															class="input-bordered input input-sm w-full max-w-xs font-mono"
															bind:value={editValue}
															onkeydown={(e) => {
																if (e.key === 'Enter') saveSetting(addon.name, setting.key);
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
																class="btn btn-xs btn-success"
																disabled={saving}
																onclick={() => saveSetting(addon.name, setting.key)}
															>
																{#if saving}
																	<span class="loading loading-xs loading-spinner"></span>
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
				</div>
			</div>
		{/each}
	</div>
{/if}
