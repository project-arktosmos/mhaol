<script lang="ts">
	import { onMount } from 'svelte';
	import { get } from 'svelte/store';
	import classNames from 'classnames';
	import {
		librariesService,
		LIBRARY_ADDONS,
		LIBRARY_ADDON_LABELS,
		type LibraryAddon,
		type ScanResponse,
		type Library
	} from '$lib/libraries.service';
	import type { IpfsPin } from '$lib/ipfs.service';
	import DirectoryPicker from '../../components/DirectoryPicker.svelte';

	const libsStore = librariesService.state;
	const SCAN_STALE_MS = 60 * 60 * 1000;

	let pickedDir = $state('');
	let newSubfolder = $state('');
	let newAddons = $state<LibraryAddon[]>([]);
	let creating = $state(false);
	let createError = $state<string | null>(null);
	let deletingId = $state<string | null>(null);
	let scanningId = $state<string | null>(null);
	let scanResults = $state<Record<string, ScanResponse>>({});
	let scanErrors = $state<Record<string, string>>({});
	let libPins = $state<Record<string, IpfsPin[]>>({});
	let pinsErrors = $state<Record<string, string>>({});
	let editingKindsFor = $state<string | null>(null);
	let editAddons = $state<LibraryAddon[]>([]);
	let savingKinds = $state(false);
	let addonsError = $state<string | null>(null);

	function toggleNewAddon(addon: LibraryAddon) {
		newAddons = newAddons.includes(addon)
			? newAddons.filter((k) => k !== addon)
			: [...newAddons, addon];
	}

	function toggleEditAddon(addon: LibraryAddon) {
		editAddons = editAddons.includes(addon)
			? editAddons.filter((k) => k !== addon)
			: [...editAddons, addon];
	}

	function startEditKinds(lib: Library) {
		editingKindsFor = lib.id;
		editAddons = [...(lib.addons ?? [])];
		addonsError = null;
	}

	function cancelEditKinds() {
		editingKindsFor = null;
		editAddons = [];
		addonsError = null;
	}

	async function saveKinds(id: string) {
		savingKinds = true;
		addonsError = null;
		try {
			await librariesService.update(id, { addons: editAddons });
			editingKindsFor = null;
			editAddons = [];
		} catch (err) {
			addonsError = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			savingKinds = false;
		}
	}

	onMount(async () => {
		await librariesService.refresh();
		const { libraries } = get(libsStore);
		await Promise.all(libraries.map(handleLibraryOnMount));
	});

	function isStale(lib: Library): boolean {
		if (!lib.last_scanned_at) return true;
		const ts = new Date(lib.last_scanned_at).getTime();
		if (Number.isNaN(ts)) return true;
		return Date.now() - ts > SCAN_STALE_MS;
	}

	async function handleLibraryOnMount(lib: Library) {
		if (isStale(lib)) {
			await scan(lib.id);
		} else {
			await Promise.all([loadPins(lib.id), loadScanResult(lib.id)]);
		}
	}

	async function loadScanResult(id: string) {
		try {
			const result = await librariesService.lastScanResult(id);
			if (result) {
				scanResults = { ...scanResults, [id]: result };
			}
		} catch {
			// no persisted scan result yet — leave scanResults untouched
		}
	}

	async function loadPins(id: string) {
		try {
			const pins = await librariesService.pins(id);
			libPins = { ...libPins, [id]: pins };
			const { [id]: _ignored, ...rest } = pinsErrors;
			pinsErrors = rest;
		} catch (err) {
			pinsErrors = {
				...pinsErrors,
				[id]: err instanceof Error ? err.message : 'Unknown error'
			};
		}
	}

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
			const created = await librariesService.create(finalPath, newAddons);
			newSubfolder = '';
			newAddons = [];
			void scan(created.id);
		} catch (err) {
			createError = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			creating = false;
		}
	}

	const pinPollers = new Map<string, number>();

	function pollPinsUntilStable(id: string) {
		const existing = pinPollers.get(id);
		if (existing !== undefined) {
			window.clearInterval(existing);
		}
		let lastCount = libPins[id]?.length ?? 0;
		let stableTicks = 0;
		const handle = window.setInterval(async () => {
			await loadPins(id);
			const current = libPins[id]?.length ?? 0;
			if (current === lastCount) {
				stableTicks++;
				if (stableTicks >= 3) {
					window.clearInterval(handle);
					pinPollers.delete(id);
				}
			} else {
				stableTicks = 0;
				lastCount = current;
			}
		}, 3000);
		pinPollers.set(id, handle);
		window.setTimeout(() => {
			const h = pinPollers.get(id);
			if (h !== undefined) {
				window.clearInterval(h);
				pinPollers.delete(id);
			}
		}, 120000);
	}

	async function scan(id: string) {
		scanningId = id;
		const { [id]: _ignored, ...rest } = scanErrors;
		scanErrors = rest;
		try {
			const result = await librariesService.scan(id);
			scanResults = { ...scanResults, [id]: result };
			await Promise.all([librariesService.refresh(), loadPins(id)]);
			pollPinsUntilStable(id);
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

	function pinIndex(pins: IpfsPin[] | undefined): Map<string, string> {
		const map = new Map<string, string>();
		if (!pins) return map;
		for (const p of pins) map.set(p.path, p.cid);
		return map;
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
			<div class="form-control">
				<span class="label-text mb-1 text-xs">
					Kinds (which catalog types this library contains)
				</span>
				<div class="flex flex-wrap gap-2">
					{#each LIBRARY_ADDONS as kind (kind)}
						<label
							class={classNames('btn btn-sm', {
								'btn-primary': newAddons.includes(kind),
								'btn-outline': !newAddons.includes(kind)
							})}
						>
							<input
								type="checkbox"
								class="hidden"
								checked={newAddons.includes(kind)}
								disabled={creating}
								onchange={() => toggleNewAddon(kind)}
							/>
							{LIBRARY_ADDON_LABELS[kind]}
						</label>
					{/each}
				</div>
				<p class="mt-1 text-xs text-base-content/60">
					Files matching the selected kinds are pinned to IPFS on scan. Leave empty to skip pinning.
				</p>
			</div>
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
							<th>Kinds</th>
							<th>Created</th>
							<th>Last scanned</th>
							<th class="w-24"></th>
						</tr>
					</thead>
					<tbody>
						{#each $libsStore.libraries as lib (lib.id)}
							<tr>
								<td class="font-mono text-xs break-all">{lib.path}</td>
								<td class="text-xs">
									{#if editingKindsFor === lib.id}
										<div class="flex flex-wrap items-center gap-1">
											{#each LIBRARY_ADDONS as kind (kind)}
												<label
													class={classNames('btn btn-xs', {
														'btn-primary': editAddons.includes(kind),
														'btn-outline': !editAddons.includes(kind)
													})}
												>
													<input
														type="checkbox"
														class="hidden"
														checked={editAddons.includes(kind)}
														disabled={savingKinds}
														onchange={() => toggleEditAddon(kind)}
													/>
													{LIBRARY_ADDON_LABELS[kind]}
												</label>
											{/each}
											<button
												class="btn btn-xs btn-primary"
												disabled={savingKinds}
												onclick={() => saveKinds(lib.id)}
											>
												{savingKinds ? 'Saving…' : 'Save'}
											</button>
											<button
												class="btn btn-ghost btn-xs"
												disabled={savingKinds}
												onclick={cancelEditKinds}
											>
												Cancel
											</button>
										</div>
										{#if addonsError}
											<p class="mt-1 text-error">{addonsError}</p>
										{/if}
									{:else if (lib.addons ?? []).length === 0}
										<button class="btn btn-ghost btn-xs" onclick={() => startEditKinds(lib)}>
											Set kinds
										</button>
									{:else}
										<button
											class="badge flex flex-wrap gap-1 badge-outline badge-sm"
											onclick={() => startEditKinds(lib)}
										>
											{(lib.addons ?? []).map((k) => LIBRARY_ADDON_LABELS[k] ?? k).join(', ')}
										</button>
									{/if}
								</td>
								<td class="text-xs text-base-content/60">{formatDate(lib.created_at)}</td>
								<td class="text-xs text-base-content/60">
									{lib.last_scanned_at ? formatDate(lib.last_scanned_at) : '—'}
								</td>
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
									<td colspan="5" class="bg-base-100">
										<div class="my-2 alert alert-error">
											<span class="text-sm">{scanErrors[lib.id]}</span>
											<button class="btn btn-ghost btn-xs" onclick={() => clearScan(lib.id)}>
												Dismiss
											</button>
										</div>
									</td>
								</tr>
							{:else if pinsErrors[lib.id]}
								<tr>
									<td colspan="5" class="bg-base-100">
										<div class="my-2 alert alert-warning">
											<span class="text-sm">Pins: {pinsErrors[lib.id]}</span>
										</div>
									</td>
								</tr>
							{:else if scanResults[lib.id]}
								{@const cidByPath = pinIndex(libPins[lib.id])}
								<tr>
									<td colspan="5" class="bg-base-100 p-3">
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
												<div class="max-h-96 overflow-y-auto rounded border border-base-content/10">
													<table class="table table-xs">
														<thead class="sticky top-0 bg-base-200">
															<tr>
																<th>File</th>
																<th class="w-72">CID</th>
																<th class="w-24">MIME</th>
																<th class="w-20 text-right">Size</th>
																<th class="w-56">TMDB match</th>
															</tr>
														</thead>
														<tbody>
															{#each scanResults[lib.id].entries as entry (entry.path)}
																{@const cid = cidByPath.get(entry.path)}
																<tr>
																	<td class="font-mono text-xs break-all">
																		{entry.relative_path}
																	</td>
																	<td class="font-mono text-xs break-all">
																		{#if cid}
																			{cid}
																		{:else}
																			<span class="text-base-content/40">pinning…</span>
																		{/if}
																	</td>
																	<td class="font-mono text-xs">{entry.mime}</td>
																	<td class="text-right text-xs">
																		{formatBytes(entry.size)}
																	</td>
																	<td class="text-xs">
																		{#if entry.tmdbMatch}
																			<a
																				class="link link-hover"
																				href={`https://www.themoviedb.org/movie/${entry.tmdbMatch.tmdbId}`}
																				target="_blank"
																				rel="noreferrer"
																				title={entry.tmdbMatch.overview ?? ''}
																			>
																				{entry.tmdbMatch.title}{entry.tmdbMatch.year
																					? ` (${entry.tmdbMatch.year})`
																					: ''}
																			</a>
																		{:else if entry.mime.startsWith('video/')}
																			<span class="text-base-content/40">no match</span>
																		{:else}
																			<span class="text-base-content/30">—</span>
																		{/if}
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
