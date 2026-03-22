<script lang="ts">
	import type { LibraryFile } from 'ui-lib/types/library.type';
	import LibraryFileItem from './LibraryFileItem.svelte';
	import TmdbLinkModal from './TmdbLinkModal.svelte';
	import MusicBrainzLinkModal from './MusicBrainzLinkModal.svelte';
	import YouTubePreviewModal from './YouTubePreviewModal.svelte';
	import MediaTypeCategoryModal from './MediaTypeCategoryModal.svelte';

	interface Props {
		files: LibraryFile[];
		loading: boolean;
		error: string | null;
		onscan: () => void;
		onlink: (
			file: LibraryFile,
			tmdbId: number,
			seasonNumber: number | null,
			episodeNumber: number | null,
			type: 'movie' | 'tv'
		) => void;
		onunlink: (file: LibraryFile) => void;
		onyoutubelink: (file: LibraryFile, youtubeId: string) => void;
		onyoutubeunlink: (file: LibraryFile) => void;
		onmusicbrainzlink: (file: LibraryFile, musicbrainzId: string) => void;
		onmusicbrainzunlink: (file: LibraryFile) => void;
		onedittype: (file: LibraryFile, mediaType: string, categoryId: string | null) => void;
	}

	let {
		files,
		loading,
		error,
		onscan,
		onlink,
		onunlink,
		onyoutubelink,
		onyoutubeunlink,
		onmusicbrainzlink,
		onmusicbrainzunlink,
		onedittype
	}: Props = $props();

	let modalFile: LibraryFile | null = $state(null);
	let musicbrainzModalFile: LibraryFile | null = $state(null);
	let youtubePreviewFile: LibraryFile | null = $state(null);
	let typeCategoryModalFile: LibraryFile | null = $state(null);

	function openModal(file: LibraryFile) {
		modalFile = file;
	}

	function closeModal() {
		modalFile = null;
	}

	function handleLink(
		tmdbId: number,
		seasonNumber: number | null,
		episodeNumber: number | null,
		type: 'movie' | 'tv'
	) {
		if (modalFile) {
			onlink(modalFile, tmdbId, seasonNumber, episodeNumber, type);
			closeModal();
		}
	}

	function openMusicBrainzModal(file: LibraryFile) {
		musicbrainzModalFile = file;
	}

	function closeMusicBrainzModal() {
		musicbrainzModalFile = null;
	}

	function handleMusicBrainzLink(musicbrainzId: string) {
		if (musicbrainzModalFile) {
			onmusicbrainzlink(musicbrainzModalFile, musicbrainzId);
			closeMusicBrainzModal();
		}
	}

	function openYoutubePreview(file: LibraryFile) {
		youtubePreviewFile = file;
	}

	function closeYoutubePreview() {
		youtubePreviewFile = null;
	}

	function openTypeCategoryModal(file: LibraryFile) {
		typeCategoryModalFile = file;
	}

	function closeTypeCategoryModal() {
		typeCategoryModalFile = null;
	}

	function handleTypeCategorySave(mediaType: string, categoryId: string | null) {
		if (typeCategoryModalFile) {
			onedittype(typeCategoryModalFile, mediaType, categoryId);
			closeTypeCategoryModal();
		}
	}
</script>

<div class="mt-3 border-t border-base-300 pt-3">
	<div class="mb-2 flex items-center justify-between">
		<span class="text-xs text-base-content/50">
			{files.length} file{files.length !== 1 ? 's' : ''}
		</span>
		<button class="btn btn-ghost btn-xs" onclick={onscan} disabled={loading}>
			{#if loading}
				<span class="loading loading-xs loading-spinner"></span>
			{:else}
				Scan
			{/if}
		</button>
	</div>

	{#if loading && files.length === 0}
		<div class="flex justify-center py-4">
			<span class="loading loading-sm loading-spinner"></span>
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
		<div class="max-h-96 overflow-y-auto rounded-lg bg-base-100">
			<table class="table w-full table-xs">
				<thead class="sticky top-0 bg-base-100">
					<tr>
						<th>Name</th>
						<th class="w-20">Type</th>
						<th class="w-24">Category</th>
						<th class="w-20">Ext</th>
						<th class="w-24">TMDB</th>
						<th class="w-28">YouTube</th>
						<th class="w-28">MusicBrainz</th>
					</tr>
				</thead>
				<tbody>
					{#each files as file (file.path)}
						<LibraryFileItem
							{file}
							onlinkclick={openModal}
							onunlinkclick={(f) => onunlink(f)}
							onyoutubelink={(f, ytId) => onyoutubelink(f, ytId)}
							onyoutubeunlink={(f) => onyoutubeunlink(f)}
							onyoutubepreview={openYoutubePreview}
							onmusicbrainzlinkclick={openMusicBrainzModal}
							onmusicbrainzunlink={(f) => onmusicbrainzunlink(f)}
							onedittype={openTypeCategoryModal}
						/>
					{/each}
				</tbody>
			</table>
		</div>
	{/if}
</div>

{#if modalFile}
	<TmdbLinkModal file={modalFile} type="movie" onlink={handleLink} onclose={closeModal} />
{/if}

{#if musicbrainzModalFile}
	<MusicBrainzLinkModal
		file={musicbrainzModalFile}
		onlink={handleMusicBrainzLink}
		onclose={closeMusicBrainzModal}
	/>
{/if}

{#if youtubePreviewFile && youtubePreviewFile.links.youtube}
	<YouTubePreviewModal
		file={youtubePreviewFile}
		videoId={youtubePreviewFile.links.youtube.serviceId}
		onclose={closeYoutubePreview}
	/>
{/if}

{#if typeCategoryModalFile}
	<MediaTypeCategoryModal
		file={typeCategoryModalFile}
		onsave={handleTypeCategorySave}
		onclose={closeTypeCategoryModal}
	/>
{/if}
