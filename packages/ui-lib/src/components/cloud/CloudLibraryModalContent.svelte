<script lang="ts">
	import classNames from 'classnames';
	import { cloudLibraryService } from 'frontend/services/cloud-library.service';
	import { cloudPeerService } from 'frontend/services/cloud-peer.service';
	import type { DirectoryEntry } from 'frontend/types/cloud.type';

	const libStore = cloudLibraryService.store;
	const libState = cloudLibraryService.state;
	const peerState = cloudPeerService.state;

	let libraries = $derived($libStore);
	let svc = $derived($libState);
	let peers = $derived($peerState);

	let peerLibraries = $derived(
		Object.entries(peers.peers).flatMap(([peerId, data]) =>
			data.libraries.map((lib) => ({ ...lib, peerId }))
		)
	);

	let activeTab: 'libraries' | 'add' = $state('libraries');

	function switchTab(tab: 'libraries' | 'add') {
		activeTab = tab;
		if (tab === 'add') {
			cloudLibraryService.openAddForm();
		}
	}

	function handleBrowse(path: string) {
		cloudLibraryService.browseDirectory(path);
	}

	function handleSelectDir(dir: DirectoryEntry) {
		cloudLibraryService.selectDirectory(dir.path, dir.name);
	}

	async function handleConfirmAdd() {
		if (!svc.selectedPath || !svc.selectedName) return;
		await cloudLibraryService.addLibrary(svc.selectedName, svc.selectedPath);
		activeTab = 'libraries';
	}

	async function handleScan(id: string) {
		await cloudLibraryService.scanLibrary(id);
	}

	async function handleDelete(id: string) {
		await cloudLibraryService.removeLibrary(id);
	}
</script>

<div class="pr-8">
	<h3 class="text-lg font-bold">Cloud Libraries</h3>
	<p class="text-base-content/60 text-sm">Manage local and peer cloud libraries</p>
</div>

<div class="mt-4 flex gap-2">
	<div class="join">
		<button
			class={classNames('btn join-item btn-sm', {
				'btn-active': activeTab === 'libraries'
			})}
			onclick={() => switchTab('libraries')}
		>
			Libraries
		</button>
		<button
			class={classNames('btn join-item btn-sm', {
				'btn-active': activeTab === 'add'
			})}
			onclick={() => switchTab('add')}
		>
			Add Library
		</button>
	</div>
</div>

<div class="mt-4">
	{#if activeTab === 'add'}
		<div class="space-y-4">
			<div class="form-control">
				<label class="label" for="cloud-lib-name">
					<span class="label-text">Library Name</span>
				</label>
				<input
					id="cloud-lib-name"
					type="text"
					class="input input-bordered w-full"
					placeholder="My Media Library"
					value={svc.selectedName}
					oninput={(e) =>
						cloudLibraryService.setSelectedName((e.target as HTMLInputElement).value)}
				/>
			</div>

			<div class="form-control">
				<label class="label">
					<span class="label-text">Select Directory</span>
				</label>
				{#if svc.selectedPath}
					<div class="badge badge-primary badge-lg mb-2">{svc.selectedPath}</div>
				{/if}

				<div class="bg-base-100 max-h-64 overflow-y-auto rounded-lg border p-2">
					{#if svc.browseParent}
						<button
							class="btn btn-ghost btn-sm w-full justify-start"
							onclick={() => handleBrowse(svc.browseParent!)}
						>
							..
						</button>
					{/if}
					{#each svc.browseDirectories as dir (dir.path)}
						<button
							class={classNames('btn btn-ghost btn-sm w-full justify-start', {
								'btn-active': svc.selectedPath === dir.path
							})}
							onclick={() => handleSelectDir(dir)}
							ondblclick={() => handleBrowse(dir.path)}
						>
							{dir.name}
						</button>
					{/each}
				</div>
			</div>

			<div class="flex justify-end gap-2">
				<button class="btn btn-ghost" onclick={() => (activeTab = 'libraries')}>
					Cancel
				</button>
				<button
					class="btn btn-primary"
					disabled={!svc.selectedPath || !svc.selectedName}
					onclick={handleConfirmAdd}
				>
					Add
				</button>
			</div>
		</div>
	{:else}
		<div class="space-y-3">
			{#each libraries as library (library.id)}
				<div class="card bg-base-200">
					<div class="card-body p-4">
						<div class="flex items-start justify-between">
							<div class="min-w-0 flex-1">
								<h4 class="font-medium">{library.name}</h4>
								<p class="text-base-content/60 truncate text-xs">{library.path}</p>
							</div>
							<span class="badge badge-info badge-sm ml-2">Local</span>
						</div>
						<div class="mt-2 flex flex-wrap items-center gap-1">
							<span class="badge badge-neutral badge-xs">{library.itemCount} items</span>
							<span
								class={classNames('badge badge-xs', {
									'badge-success': library.scanStatus === 'idle',
									'badge-warning': library.scanStatus === 'scanning',
									'badge-error': library.scanStatus === 'error'
								})}
							>
								{library.scanStatus}
							</span>
						</div>
						{#if library.scanError}
							<p class="mt-1 text-xs text-error">{library.scanError}</p>
						{/if}
						<div class="mt-2 flex justify-end gap-2">
							<button
								class="btn btn-secondary btn-xs"
								disabled={svc.itemsLoading[library.id]}
								onclick={() => handleScan(library.id)}
							>
								{svc.itemsLoading[library.id] ? 'Scanning...' : 'Scan'}
							</button>
							<button
								class="btn btn-error btn-xs btn-outline"
								onclick={() => handleDelete(library.id)}
							>
								Delete
							</button>
						</div>
					</div>
				</div>
			{/each}

			{#each peerLibraries as peerLib (peerLib.peerId + '-' + peerLib.id)}
				<div class="card bg-base-200">
					<div class="card-body p-4">
						<div class="flex items-start justify-between">
							<div class="min-w-0 flex-1">
								<h4 class="font-medium">{peerLib.name}</h4>
								<p class="text-base-content/60 truncate text-xs">
									via {peerLib.peerId.slice(0, 8)}...
								</p>
							</div>
							<span class="badge badge-warning badge-sm ml-2">Peer</span>
						</div>
						<div class="mt-2 flex flex-wrap items-center gap-1">
							<span class="badge badge-neutral badge-xs">{peerLib.itemCount} items</span>
							<span class="badge badge-ghost badge-xs">{peerLib.kind}</span>
						</div>
					</div>
				</div>
			{/each}

			{#if libraries.length === 0 && peerLibraries.length === 0}
				<div class="py-8 text-center">
					<p class="text-base-content/60">No cloud libraries yet</p>
					<button class="btn btn-primary btn-sm mt-3" onclick={() => switchTab('add')}>
						Add your first library
					</button>
				</div>
			{/if}
		</div>
	{/if}
</div>
