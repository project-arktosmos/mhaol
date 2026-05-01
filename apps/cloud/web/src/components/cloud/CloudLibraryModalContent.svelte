<script lang="ts">
	import classNames from 'classnames';
	import { cloudLibraryService } from '$services/cloud-library.service';
	import { cloudPeerService } from '$services/cloud-peer.service';
	import type { DirectoryEntry } from '$types/cloud.type';

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
	<p class="text-sm text-base-content/60">Manage local and peer cloud libraries</p>
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
					class="input-bordered input w-full"
					placeholder="My Media Library"
					value={svc.selectedName}
					oninput={(e) => cloudLibraryService.setSelectedName((e.target as HTMLInputElement).value)}
				/>
			</div>

			<div class="form-control">
				<label class="label">
					<span class="label-text">Select Directory</span>
				</label>
				{#if svc.selectedPath}
					<div class="mb-2 badge badge-lg badge-primary">{svc.selectedPath}</div>
				{/if}

				<div class="max-h-64 overflow-y-auto rounded-lg border bg-base-100 p-2">
					{#if svc.browseParent}
						<button
							class="btn w-full justify-start btn-ghost btn-sm"
							onclick={() => handleBrowse(svc.browseParent!)}
						>
							..
						</button>
					{/if}
					{#each svc.browseDirectories as dir (dir.path)}
						<button
							class={classNames('btn w-full justify-start btn-ghost btn-sm', {
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
				<button class="btn btn-ghost" onclick={() => (activeTab = 'libraries')}> Cancel </button>
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
								<p class="truncate text-xs text-base-content/60">{library.path}</p>
							</div>
							<span class="ml-2 badge badge-sm badge-info">Local</span>
						</div>
						<div class="mt-2 flex flex-wrap items-center gap-1">
							<span class="badge badge-xs badge-neutral">{library.itemCount} items</span>
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
								class="btn btn-xs btn-secondary"
								disabled={svc.itemsLoading[library.id]}
								onclick={() => handleScan(library.id)}
							>
								{svc.itemsLoading[library.id] ? 'Scanning...' : 'Scan'}
							</button>
							<button
								class="btn btn-outline btn-xs btn-error"
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
								<p class="truncate text-xs text-base-content/60">
									via {peerLib.peerId.slice(0, 8)}...
								</p>
							</div>
							<span class="ml-2 badge badge-sm badge-warning">Peer</span>
						</div>
						<div class="mt-2 flex flex-wrap items-center gap-1">
							<span class="badge badge-xs badge-neutral">{peerLib.itemCount} items</span>
							<span class="badge badge-ghost badge-xs">{peerLib.kind}</span>
						</div>
					</div>
				</div>
			{/each}

			{#if libraries.length === 0 && peerLibraries.length === 0}
				<div class="py-8 text-center">
					<p class="text-base-content/60">No cloud libraries yet</p>
					<button class="btn mt-3 btn-sm btn-primary" onclick={() => switchTab('add')}>
						Add your first library
					</button>
				</div>
			{/if}
		</div>
	{/if}
</div>
