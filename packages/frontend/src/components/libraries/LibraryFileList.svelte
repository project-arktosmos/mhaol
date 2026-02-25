<script lang="ts">
	import type { LibraryFile } from '$types/library.type';
	import LibraryFileItem from './LibraryFileItem.svelte';

	interface Props {
		files: LibraryFile[];
		loading: boolean;
		error: string | null;
		onrefresh: () => void;
	}

	let { files, loading, error, onrefresh }: Props = $props();
</script>

<div class="mt-3 border-t border-base-300 pt-3">
	<div class="mb-2 flex items-center justify-between">
		<span class="text-xs text-base-content/50">
			{files.length} file{files.length !== 1 ? 's' : ''}
		</span>
		<button class="btn btn-ghost btn-xs" onclick={onrefresh} disabled={loading}>
			{#if loading}
				<span class="loading loading-spinner loading-xs"></span>
			{:else}
				Refresh
			{/if}
		</button>
	</div>

	{#if loading && files.length === 0}
		<div class="flex justify-center py-4">
			<span class="loading loading-spinner loading-sm"></span>
		</div>
	{:else if error}
		<div class="rounded-lg bg-error/10 px-3 py-2 text-sm text-error">
			{error}
		</div>
	{:else if files.length === 0}
		<div class="rounded-lg bg-base-300 py-4 text-center">
			<p class="text-sm opacity-50">No media files found</p>
		</div>
	{:else}
		<div class="flex max-h-64 flex-col gap-1 overflow-y-auto">
			{#each files as file (file.path)}
				<LibraryFileItem {file} />
			{/each}
		</div>
	{/if}
</div>
