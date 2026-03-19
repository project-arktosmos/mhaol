<script lang="ts">
	import type { YouTubeContent } from '$types/youtube.type';
	import LibraryFileItem from './LibraryFileItem.svelte';

	interface Props {
		content: YouTubeContent[];
		loading: boolean;
		error: string | null;
	}

	let { content, loading, error }: Props = $props();
</script>

<div class="mt-3 border-t border-base-300 pt-3">
	<div class="mb-2 flex items-center justify-between">
		<span class="text-xs text-base-content/50">
			{content.length} item{content.length !== 1 ? 's' : ''}
		</span>
	</div>

	{#if loading && content.length === 0}
		<div class="flex justify-center py-4">
			<span class="loading loading-sm loading-spinner"></span>
		</div>
	{:else if error}
		<div class="rounded-lg bg-error/10 px-3 py-2 text-sm text-error">
			{error}
		</div>
	{:else if content.length === 0}
		<div class="rounded-lg bg-base-300 py-4 text-center">
			<p class="text-sm opacity-50">No downloaded content yet</p>
		</div>
	{:else}
		<div class="max-h-96 overflow-y-auto rounded-lg bg-base-100">
			<table class="table w-full table-xs">
				<thead class="sticky top-0 bg-base-100">
					<tr>
						<th class="w-12"></th>
						<th>Title</th>
						<th class="w-20">Duration</th>
						<th class="w-24">Play</th>
					</tr>
				</thead>
				<tbody>
					{#each content as item (item.youtubeId)}
						<LibraryFileItem {item} />
					{/each}
				</tbody>
			</table>
		</div>
	{/if}
</div>
