<script lang="ts">
	import classNames from 'classnames';
	import Modal from '$components/core/Modal.svelte';
	import { diskModalService } from '$services/disk-modal.service';
	import { diskService, type DiskInfo, type SubdirInfo } from '$lib/disk.service';

	const store = diskService.state;
	const modalStore = diskModalService.store;
	let firstOpenSeen = false;

	$effect(() => {
		if ($modalStore.open && !firstOpenSeen) {
			firstOpenSeen = true;
			void diskService.refresh();
		}
	});

	function close() {
		diskModalService.close();
	}

	function formatBytes(bytes: number): string {
		if (bytes < 1024) return `${bytes} B`;
		const units = ['KB', 'MB', 'GB', 'TB', 'PB'];
		let value = bytes / 1024;
		let i = 0;
		while (value >= 1024 && i < units.length - 1) {
			value /= 1024;
			i++;
		}
		return `${value.toFixed(value >= 100 ? 0 : value >= 10 ? 1 : 2)} ${units[i]}`;
	}

	function usedPct(d: DiskInfo): number {
		if (d.totalBytes === 0) return 0;
		return Math.min(100, (d.usedBytes / d.totalBytes) * 100);
	}

	function subdirPct(s: SubdirInfo, total: number): number {
		if (total === 0) return 0;
		return Math.min(100, (s.sizeBytes / total) * 100);
	}

	function pctBarColor(pct: number): string {
		if (pct >= 90) return 'progress-error';
		if (pct >= 75) return 'progress-warning';
		return 'progress-primary';
	}
</script>

<Modal open={$modalStore.open} maxWidth="max-w-6xl" onclose={close}>
	<div class="flex flex-col gap-6">
		<header class="flex items-start justify-between gap-4">
			<div>
				<h2 class="text-2xl font-bold">Disk</h2>
				<p class="text-sm text-base-content/60">
					Storage volumes mounted on this machine, plus a per-subdirectory breakdown of the cloud's
					data root.
				</p>
			</div>
			<button
				class="btn btn-outline btn-sm"
				onclick={() => diskService.refresh()}
				disabled={$store.loading}
			>
				{$store.loading ? 'Refreshing…' : 'Refresh'}
			</button>
		</header>

		{#if $store.error}
			<div class="alert alert-error">
				<span>{$store.error}</span>
			</div>
		{/if}

		{#if $store.loading && !$store.data}
			<p class="text-sm text-base-content/60">Loading…</p>
		{:else if $store.data}
			{@const data = $store.data}
			<section class="flex flex-col gap-3">
				<h3 class="text-lg font-semibold">Volumes ({data.disks.length})</h3>
				{#if data.disks.length === 0}
					<p class="text-sm text-base-content/60">No mounted volumes reported.</p>
				{:else}
					<div class="overflow-x-auto rounded-box border border-base-content/10">
						<table class="table table-sm">
							<thead>
								<tr>
									<th>Mount</th>
									<th>Name</th>
									<th>FS</th>
									<th>Kind</th>
									<th class="w-24 text-right">Total</th>
									<th class="w-24 text-right">Available</th>
									<th class="w-24 text-right">Used</th>
									<th class="w-64">Usage</th>
								</tr>
							</thead>
							<tbody>
								{#each data.disks as disk (disk.mountPoint + ':' + disk.name)}
									{@const pct = usedPct(disk)}
									<tr
										class={classNames({
											'bg-primary/5': disk.isDataRootDisk
										})}
									>
										<td class="font-mono text-xs break-all">
											{disk.mountPoint}
											{#if disk.isDataRootDisk}
												<span class="ml-2 badge badge-sm badge-primary">data root</span>
											{/if}
											{#if disk.isRemovable}
												<span class="ml-1 badge badge-ghost badge-sm">removable</span>
											{/if}
										</td>
										<td class="font-mono text-xs break-all">{disk.name || '—'}</td>
										<td class="font-mono text-xs">{disk.fileSystem || '—'}</td>
										<td class="font-mono text-xs">{disk.kind}</td>
										<td class="text-right text-xs">{formatBytes(disk.totalBytes)}</td>
										<td class="text-right text-xs">{formatBytes(disk.availableBytes)}</td>
										<td class="text-right text-xs">{formatBytes(disk.usedBytes)}</td>
										<td>
											<div class="flex items-center gap-2">
												<progress
													class={classNames('progress w-40', pctBarColor(pct))}
													value={pct}
													max="100"
												></progress>
												<span class="font-mono text-xs text-base-content/70">
													{pct.toFixed(1)}%
												</span>
											</div>
										</td>
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				{/if}
			</section>

			<section class="flex flex-col gap-3">
				<div class="flex items-baseline justify-between gap-3">
					<h3 class="text-lg font-semibold">Data root breakdown</h3>
					<span class="text-xs text-base-content/60">
						Total: <span class="font-mono">{formatBytes(data.dataRootTotalBytes)}</span>
					</span>
				</div>
				<p class="text-sm text-base-content/60">
					Path: <span class="font-mono">{data.dataRoot}</span>
					{#if data.dataRootMountPoint}
						· On volume <span class="font-mono">{data.dataRootMountPoint}</span>
					{/if}
				</p>

				<div class="overflow-x-auto rounded-box border border-base-content/10">
					<table class="table table-sm">
						<thead>
							<tr>
								<th>Subdir</th>
								<th>Path</th>
								<th class="w-16">Kind</th>
								<th class="w-24 text-right">Size</th>
								<th class="w-64">Share</th>
							</tr>
						</thead>
						<tbody>
							{#each data.subdirs as sub (sub.path)}
								{@const pct = subdirPct(sub, data.dataRootTotalBytes)}
								<tr class={classNames({ 'opacity-50': !sub.exists })}>
									<td class="font-mono text-xs">{sub.name}</td>
									<td class="font-mono text-xs break-all">{sub.path}</td>
									<td class="font-mono text-xs">
										{sub.kind}{#if !sub.exists}
											<span class="text-base-content/50">(missing)</span>{/if}
									</td>
									<td class="text-right text-xs">{formatBytes(sub.sizeBytes)}</td>
									<td>
										<div class="flex items-center gap-2">
											<progress class="progress w-40 progress-secondary" value={pct} max="100"
											></progress>
											<span class="font-mono text-xs text-base-content/70">
												{pct.toFixed(1)}%
											</span>
										</div>
									</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			</section>
		{/if}
	</div>
</Modal>
