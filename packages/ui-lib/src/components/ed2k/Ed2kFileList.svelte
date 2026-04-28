<script lang="ts">
	import classNames from 'classnames';
	import { ed2kService } from 'ui-lib/services/ed2k.service';
	import { ed2kStateLabel, ed2kStateColor } from 'ui-lib/types/ed2k.type';

	const state = ed2kService.state;

	function formatSize(bytes: number): string {
		if (bytes <= 0) return '0 B';
		const units = ['B', 'KB', 'MB', 'GB', 'TB'];
		const i = Math.floor(Math.log(bytes) / Math.log(1024));
		const v = bytes / Math.pow(1024, i);
		return `${v.toFixed(i === 0 ? 0 : 1)} ${units[i]}`;
	}

	function formatSpeed(bytesPerSec: number): string {
		if (bytesPerSec <= 0) return '0 B/s';
		return `${formatSize(bytesPerSec)}/s`;
	}

	function formatEta(seconds: number | null | undefined): string {
		if (!seconds || seconds <= 0 || !Number.isFinite(seconds)) return '—';
		if (seconds < 60) return `${Math.round(seconds)}s`;
		if (seconds < 3600) return `${Math.floor(seconds / 60)}m ${Math.round(seconds % 60)}s`;
		const h = Math.floor(seconds / 3600);
		const m = Math.floor((seconds % 3600) / 60);
		if (h < 24) return `${h}h ${m}m`;
		const d = Math.floor(h / 24);
		return `${d}d ${h % 24}h`;
	}

	function handlePause(hash: string) {
		ed2kService.pauseFile(hash);
	}
	function handleResume(hash: string) {
		ed2kService.resumeFile(hash);
	}
	function handleRemove(hash: string) {
		ed2kService.removeFile(hash);
	}
	function handleRemoveAll() {
		ed2kService.removeAll();
	}
</script>

<div class="card bg-base-200">
	<div class="card-body">
		<div class="flex items-center justify-between">
			<h2 class="card-title text-lg">Files</h2>
			{#if $state.files.length > 0}
				<button class="btn btn-ghost btn-sm" onclick={handleRemoveAll}>Remove All</button>
			{/if}
		</div>

		{#if $state.files.length === 0}
			<div class="py-6 text-center text-base-content/50">
				<p>No ed2k files</p>
				<p class="text-sm">Add an ed2k:// link or use search</p>
			</div>
		{:else}
			<div class="flex flex-col gap-2">
				{#each $state.files as file (file.fileHash)}
					{@const color = ed2kStateColor(file.state)}
					{@const pct = Math.max(0, Math.min(100, (file.progress ?? 0) * 100))}
					{@const downloaded = Math.floor((file.progress ?? 0) * file.size)}
					<div class="rounded-lg bg-base-100 p-3">
						<div class="flex items-start justify-between gap-3">
							<div class="flex-1 overflow-hidden">
								<h3 class="truncate font-medium" title={file.name}>{file.name}</h3>
								<div class="mt-1 flex flex-wrap items-center gap-2">
									<span
										class={classNames('badge badge-sm', {
											'badge-info': color === 'info',
											'badge-primary': color === 'primary',
											'badge-success': color === 'success',
											'badge-warning': color === 'warning',
											'badge-error': color === 'error',
											'badge-neutral': color === 'neutral'
										})}
									>
										{ed2kStateLabel(file.state)}
									</span>
									<span class="text-xs text-base-content/60">{formatSize(file.size)}</span>
									<span class="font-mono text-xs text-base-content/40">{file.fileHash}</span>
								</div>
								{#if file.outputPath}
									<p class="mt-1 truncate text-xs text-base-content/50" title={file.outputPath}>
										{file.outputPath}
									</p>
								{/if}
							</div>
							<div class="flex items-center gap-1">
								{#if file.state === 'paused' || file.state === 'error'}
									<button
										class="btn btn-ghost btn-xs"
										onclick={() => handleResume(file.fileHash)}
										title="Resume">Resume</button
									>
								{:else}
									<button
										class="btn btn-ghost btn-xs"
										onclick={() => handlePause(file.fileHash)}
										title="Pause">Pause</button
									>
								{/if}
								<button
									class="btn text-error btn-ghost btn-xs"
									onclick={() => handleRemove(file.fileHash)}
									title="Remove">Remove</button
								>
							</div>
						</div>

						<div class="mt-2 flex items-center gap-2">
							<progress
								class={classNames('progress h-2 flex-1', {
									'progress-info': color === 'info',
									'progress-primary': color === 'primary',
									'progress-success': color === 'success',
									'progress-warning': color === 'warning',
									'progress-error': color === 'error'
								})}
								value={pct}
								max="100"
							></progress>
							<span class="w-12 text-right font-mono text-xs tabular-nums">{pct.toFixed(1)}%</span>
						</div>

						<div class="mt-1 flex flex-wrap items-center gap-x-4 gap-y-1 text-xs text-base-content/60">
							<span>{formatSize(downloaded)} / {formatSize(file.size)}</span>
							<span>{formatSpeed(file.downloadSpeed)}</span>
							<span>ETA {formatEta(file.eta)}</span>
							<span>{file.peers} peers · {file.seeds} seeds</span>
						</div>
					</div>
				{/each}
			</div>
		{/if}
	</div>
</div>
