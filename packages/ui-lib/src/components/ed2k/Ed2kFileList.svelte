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
					</div>
				{/each}
			</div>
		{/if}
	</div>
</div>
