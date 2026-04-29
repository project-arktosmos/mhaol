<script lang="ts">
	import { onMount } from 'svelte';
	import classNames from 'classnames';
	import { librariesService, type ScanResponse } from '$lib/libraries.service';
	import DirectoryPicker from '../../components/DirectoryPicker.svelte';

	const libsStore = librariesService.state;

	let pickedDir = $state('');
	let newSubfolder = $state('');
	let creating = $state(false);
	let createError = $state<string | null>(null);
	let deletingId = $state<string | null>(null);
	let scanningId = $state<string | null>(null);
	let scanResults = $state<Record<string, ScanResponse>>({});
	let scanErrors = $state<Record<string, string>>({});

	onMount(() => {
		librariesService.refresh();
	});

	function sanitize(value: string): string {
		// eslint-disable-next-line no-control-regex
		const controlAndIllegal = /[\\/:*?"<>|\x00-\x1f]/g;
		return value.replace(controlAndIllegal, '_').trim();
	}

	function joinPath(base: string, child: string): string {
		if (!base) return child;
		const sep = base.includes('\\') && !base.includes('/') ? '\\' : '/';
		const trimmed = base.endsWith('/') || base.endsWith('\\') ? base.slice(0, -1) : base;
		return `${trimmed}${sep}${child}`;
	}

	const finalPath = $derived.by(() => {
		const sub = sanitize(newSubfolder);
		if (!pickedDir) return '';
		if (!sub) return pickedDir;
		return joinPath(pickedDir, sub);
	});

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		createError = null;
		if (!finalPath) {
			createError = 'Pick a directory';
			return;
		}
		creating = true;
		try {
			await librariesService.create(finalPath);
			newSubfolder = '';
		} catch (err) {
			createError = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			creating = false;
		}
	}

	async function scan(id: string) {
		scanningId = id;
		const { [id]: _ignored, ...rest } = scanErrors;
		scanErrors = rest;
		try {
			const result = await librariesService.scan(id);
			scanResults = { ...scanResults, [id]: result };
		} catch (err) {
			scanErrors = {
				...scanErrors,
				[id]: err instanceof Error ? err.message : 'Unknown error'
			};
		} finally {
			scanningId = null;
		}
	}

	function clearScan(id: string) {
		const { [id]: _ignored, ...rest } = scanResults;
		scanResults = rest;
		const { [id]: _ignored2, ...errRest } = scanErrors;
		scanErrors = errRest;
	}

	function formatBytes(bytes: number): string {
		if (bytes < 1024) return `${bytes} B`;
		const units = ['KB', 'MB', 'GB', 'TB'];
		let value = bytes / 1024;
		let i = 0;
		while (value >= 1024 && i < units.length - 1) {
			value /= 1024;
			i++;
		}
		return `${value.toFixed(value >= 100 ? 0 : value >= 10 ? 1 : 2)} ${units[i]}`;
	}

	async function remove(id: string) {
		deletingId = id;
		try {
			await librariesService.remove(id);
		} catch (err) {
			librariesService.state.update((s) => ({
				...s,
				error: err instanceof Error ? err.message : 'Unknown error'
			}));
		} finally {
			deletingId = null;
		}
	}

	function formatDate(value: string): string {
		try {
			return new Date(value).toLocaleString();
		} catch {
			return value;
		}
	}
</script>

<svelte:head>
	<title>Mhaol Cloud — Libraries</title>
</svelte:head>

<div class="flex min-h-full flex-col gap-6 p-6">
	<header class="flex items-center justify-between gap-4">
		<div>
			<h1 class="text-2xl font-bold">Libraries</h1>
			<p class="text-sm text-base-content/60">
				Each library is a directory on this machine. Browse to an existing folder to use it as a
				library, or pick a parent and create a new subfolder.
			</p>
		</div>
		<button
			class="btn btn-outline btn-sm"
			onclick={() => librariesService.refresh()}
			disabled={$libsStore.loading}
		>
			Refresh
		</button>
	</header>

	{#if $libsStore.error}
		<div class="alert alert-error">
			<span>{$libsStore.error}</span>
		</div>
	{/if}

	<section class="card border border-base-content/10 bg-base-200 p-4">
		<h2 class="mb-3 text-lg font-semibold">Add a library</h2>
		<form class="flex flex-col gap-3" onsubmit={submit}>
			<div class="form-control">
				<span class="label-text mb-1 text-xs">Directory</span>
				<DirectoryPicker value={pickedDir} disabled={creating} onChange={(p) => (pickedDir = p)} />
			</div>
			<label class="form-control">
				<span class="label-text text-xs">
					New subfolder (optional — created inside the picked directory)
				</span>
				<input
					type="text"
					class="input-bordered input input-sm"
					placeholder="leave empty to use the picked folder"
					bind:value={newSubfolder}
					disabled={creating}
				/>
			</label>
			<p class="text-xs text-base-content/60">
				Library directory: <span class="font-mono">{finalPath || '—'}</span>
			</p>
			<div>
				<button
					type="submit"
					class={classNames('btn btn-sm btn-primary', {
						'btn-disabled': creating || !finalPath
					})}
					disabled={creating || !finalPath}
				>
					{creating ? 'Creating…' : 'Create'}
				</button>
			</div>
		</form>
		{#if createError}
			<p class="mt-2 text-sm text-error">{createError}</p>
		{/if}
	</section>

	<section class="flex flex-col gap-3">
		<h2 class="text-lg font-semibold">Existing libraries</h2>
		{#if $libsStore.loading && $libsStore.libraries.length === 0}
			<p class="text-sm text-base-content/60">Loading…</p>
		{:else if $libsStore.libraries.length === 0}
			<p class="text-sm text-base-content/60">No libraries yet.</p>
		{:else}
			<div class="overflow-x-auto rounded-box border border-base-content/10">
				<table class="table table-sm">
					<thead>
						<tr>
							<th>Path</th>
							<th>Created</th>
							<th class="w-24"></th>
						</tr>
					</thead>
					<tbody>
						{#each $libsStore.libraries as lib (lib.id)}
							<tr>
								<td class="font-mono text-xs break-all">{lib.path}</td>
								<td class="text-xs text-base-content/60">{formatDate(lib.created_at)}</td>
								<td class="text-right">
									<div class="flex justify-end gap-1">
										<button
											class="btn btn-ghost btn-xs"
											onclick={() => scan(lib.id)}
											disabled={scanningId === lib.id}
										>
											{scanningId === lib.id ? 'Scanning…' : 'Scan'}
										</button>
										<button
											class="btn text-error btn-ghost btn-xs"
											onclick={() => remove(lib.id)}
											disabled={deletingId === lib.id}
										>
											{deletingId === lib.id ? 'Removing…' : 'Remove'}
										</button>
									</div>
								</td>
							</tr>
							{#if scanErrors[lib.id]}
								<tr>
									<td colspan="3" class="bg-base-100">
										<div class="my-2 alert alert-error">
											<span class="text-sm">{scanErrors[lib.id]}</span>
											<button class="btn btn-ghost btn-xs" onclick={() => clearScan(lib.id)}>
												Dismiss
											</button>
										</div>
									</td>
								</tr>
							{:else if scanResults[lib.id]}
								<tr>
									<td colspan="3" class="bg-base-100 p-3">
										<div class="flex flex-col gap-2">
											<div class="flex items-center justify-between gap-2">
												<p class="text-xs text-base-content/70">
													{scanResults[lib.id].total_files} files —
													{formatBytes(scanResults[lib.id].total_size)} total
												</p>
												<button class="btn btn-ghost btn-xs" onclick={() => clearScan(lib.id)}>
													Hide
												</button>
											</div>
											{#if scanResults[lib.id].entries.length === 0}
												<p class="text-xs text-base-content/60">No files in this directory.</p>
											{:else}
												<div class="max-h-72 overflow-y-auto rounded border border-base-content/10">
													<table class="table table-xs">
														<thead class="sticky top-0 bg-base-200">
															<tr>
																<th>Path</th>
																<th class="w-32">MIME</th>
																<th class="w-24 text-right">Size</th>
															</tr>
														</thead>
														<tbody>
															{#each scanResults[lib.id].entries as entry (entry.path)}
																<tr>
																	<td class="font-mono text-xs break-all">
																		{entry.relative_path}
																	</td>
																	<td class="font-mono text-xs">{entry.mime}</td>
																	<td class="text-right text-xs">
																		{formatBytes(entry.size)}
																	</td>
																</tr>
															{/each}
														</tbody>
													</table>
												</div>
											{/if}
										</div>
									</td>
								</tr>
							{/if}
						{/each}
					</tbody>
				</table>
			</div>
		{/if}
	</section>
</div>
