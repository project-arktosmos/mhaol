<script lang="ts">
	import classNames from 'classnames';
	import type { LibraryFile } from '$types/library.type';
	import { MediaType } from '$types/library.type';
	import { libraryFileAdapter } from '$adapters/classes/library-file.adapter';

	interface Props {
		file: LibraryFile;
	}

	let { file }: Props = $props();
</script>

<div class="flex items-center gap-3 rounded-lg bg-base-100 px-3 py-2">
	<div class="flex h-8 w-8 flex-shrink-0 items-center justify-center rounded bg-base-300">
		<svg
			xmlns="http://www.w3.org/2000/svg"
			class="h-4 w-4 opacity-40"
			fill="none"
			viewBox="0 0 24 24"
			stroke="currentColor"
			stroke-width="2"
		>
			{#if file.mediaType === MediaType.Video}
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z"
				/>
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
				/>
			{:else if file.mediaType === MediaType.Music}
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3"
				/>
			{:else}
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"
				/>
			{/if}
		</svg>
	</div>

	<div class="flex-1 min-w-0">
		<p class="truncate text-sm font-medium" title={file.name}>{file.name}</p>
		<div class="flex items-center gap-2">
			<span
				class={classNames(
					'badge badge-xs',
					libraryFileAdapter.getMediaTypeBadgeClass(file.mediaType)
				)}
			>
				{libraryFileAdapter.getMediaTypeLabel(file.mediaType)}
			</span>
			<span class="text-xs uppercase opacity-60">{file.extension}</span>
			<span class="text-xs opacity-60">{libraryFileAdapter.formatSize(file.size)}</span>
		</div>
	</div>
</div>
