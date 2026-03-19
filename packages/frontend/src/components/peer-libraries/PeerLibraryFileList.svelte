<script lang="ts">
	import classNames from 'classnames';
	import type { PeerLibraryFileInfo } from 'frontend/types/peer-library.type';

	let {
		files,
		loading
	}: {
		files: PeerLibraryFileInfo[];
		loading: boolean;
	} = $props();

	function extensionBadgeClass(ext: string): string {
		const video = ['mp4', 'mkv', 'avi', 'mov', 'wmv', 'webm', 'flv', 'm4v'];
		if (video.includes(ext)) return 'badge-primary';
		return 'badge-neutral';
	}
</script>

{#if loading}
	<div class="flex items-center justify-center py-4">
		<span class="loading loading-sm loading-spinner"></span>
		<span class="ml-2 text-sm opacity-60">Loading files...</span>
	</div>
{:else if files.length === 0}
	<p class="py-2 text-sm opacity-50">No files in this library.</p>
{:else}
	<div class="max-h-64 overflow-y-auto">
		<table class="table w-full table-xs">
			<thead class="sticky top-0 bg-base-100">
				<tr>
					<th>Name</th>
					<th>Extension</th>
					<th>Type</th>
				</tr>
			</thead>
			<tbody>
				{#each files as file (file.id)}
					<tr>
						<td class="max-w-xs truncate">{file.name}</td>
						<td>
							<span class={classNames('badge badge-sm', extensionBadgeClass(file.extension))}>
								{file.extension}
							</span>
						</td>
						<td class="text-xs opacity-60">{file.mediaType}</td>
					</tr>
				{/each}
			</tbody>
		</table>
	</div>
{/if}
