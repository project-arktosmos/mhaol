<script lang="ts">
	import classNames from 'classnames';
	import type { Library, LibraryFile } from '$types/library.type';
	import { MediaType } from '$types/library.type';
	import LibraryFileList from './LibraryFileList.svelte';

	interface Props {
		library: Library;
		files: LibraryFile[];
		filesLoading: boolean;
		filesError: string | null;
		onremove: (library: Library) => void;
		onscan: (library: Library) => void;
		onlink: (file: LibraryFile, tmdbId: number, seasonNumber: number | null, episodeNumber: number | null) => void;
		onunlink: (file: LibraryFile) => void;
		onyoutubelink: (file: LibraryFile, youtubeId: string) => void;
		onyoutubeunlink: (file: LibraryFile) => void;
		onmusicbrainzlink: (file: LibraryFile, musicbrainzId: string) => void;
		onmusicbrainzunlink: (file: LibraryFile) => void;
		onedittype: (file: LibraryFile, mediaType: string, categoryId: string | null) => void;
	}

	let { library, files, filesLoading, filesError, onremove, onscan, onlink, onunlink, onyoutubelink, onyoutubeunlink, onmusicbrainzlink, onmusicbrainzunlink, onedittype }: Props = $props();

	const mediaTypeBadge: Record<MediaType, string> = {
		[MediaType.Video]: 'badge-primary',
		[MediaType.Image]: 'badge-secondary',
		[MediaType.Audio]: 'badge-accent'
	};

	const mediaTypeLabel: Record<MediaType, string> = {
		[MediaType.Video]: 'Video',
		[MediaType.Image]: 'Image',
		[MediaType.Audio]: 'Audio'
	};
</script>

<div class="rounded-lg bg-base-100 p-4">
	<div class="flex items-center gap-4">
		<div class="flex-1 min-w-0">
			<div class="flex items-center gap-2">
				<h3 class="font-semibold truncate">{library.name}</h3>
				<div class="flex gap-1">
					{#each library.mediaTypes as type (type)}
						<span class={classNames('badge badge-sm', mediaTypeBadge[type])}>
							{mediaTypeLabel[type]}
						</span>
					{/each}
				</div>
			</div>
			<p class="text-xs text-base-content/50 truncate font-mono mt-1">{library.path}</p>
		</div>

		<button
			class="btn btn-ghost btn-sm text-error"
			onclick={() => onremove(library)}
			title="Remove library"
		>
			<svg
				xmlns="http://www.w3.org/2000/svg"
				class="h-4 w-4"
				fill="none"
				viewBox="0 0 24 24"
				stroke="currentColor"
				stroke-width="2"
			>
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
				/>
			</svg>
		</button>
	</div>

	<LibraryFileList
		{files}
		loading={filesLoading}
		error={filesError}
		onscan={() => onscan(library)}
		{onlink}
		{onunlink}
		{onyoutubelink}
		{onyoutubeunlink}
		{onmusicbrainzlink}
		{onmusicbrainzunlink}
		{onedittype}
	/>
</div>
