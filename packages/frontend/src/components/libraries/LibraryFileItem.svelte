<script lang="ts">
	import classNames from 'classnames';
	import type { LibraryFile } from '$types/library.type';
	import { libraryFileAdapter } from '$adapters/classes/library-file.adapter';

	interface Props {
		file: LibraryFile;
		onlinkclick: (file: LibraryFile) => void;
		onunlinkclick: (file: LibraryFile) => void;
		onyoutubelink: (file: LibraryFile, youtubeId: string) => void;
		onyoutubeunlink: (file: LibraryFile) => void;
		onyoutubepreview: (file: LibraryFile) => void;
		onmusicbrainzlinkclick: (file: LibraryFile) => void;
		onmusicbrainzunlink: (file: LibraryFile) => void;
		onedittype: (file: LibraryFile) => void;
	}

	let {
		file,
		onlinkclick,
		onunlinkclick,
		onyoutubelink,
		onyoutubeunlink,
		onyoutubepreview,
		onmusicbrainzlinkclick,
		onmusicbrainzunlink,
		onedittype
	}: Props = $props();

	let tmdbLink = $derived(file.links.tmdb ?? null);
	let tmdbLabel = $derived.by(() => {
		if (!tmdbLink) return '';
		let label = `TMDB ${tmdbLink.serviceId}`;
		if (tmdbLink.seasonNumber != null) label += ` S${tmdbLink.seasonNumber}`;
		if (tmdbLink.episodeNumber != null) label += `E${tmdbLink.episodeNumber}`;
		return label;
	});

	let editingYoutube = $state(false);
	let youtubeInput = $state('');

	function startYoutubeEdit() {
		youtubeInput = '';
		editingYoutube = true;
	}

	function submitYoutubeId() {
		const trimmed = youtubeInput.trim();
		if (trimmed) {
			onyoutubelink(file, trimmed);
		}
		editingYoutube = false;
	}

	function cancelYoutubeEdit() {
		editingYoutube = false;
	}

	function handleYoutubeKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter') {
			submitYoutubeId();
		} else if (event.key === 'Escape') {
			cancelYoutubeEdit();
		}
	}
</script>

<tr class="hover">
	<td class="max-w-0">
		<span class="block truncate" title={file.path}>{file.name}</span>
	</td>
	<td>
		<button
			class="btn px-0 btn-ghost btn-xs"
			title="Edit type & category"
			onclick={() => onedittype(file)}
		>
			<span
				class={classNames(
					'badge badge-xs',
					libraryFileAdapter.getMediaTypeBadgeClass(file.mediaType)
				)}
			>
				{libraryFileAdapter.getMediaTypeLabel(file.mediaType)}
			</span>
		</button>
	</td>
	<td>
		{#if file.categoryId}
			<button
				class="btn px-0 btn-ghost btn-xs"
				title="Edit type & category"
				onclick={() => onedittype(file)}
			>
				<span
					class={classNames(
						'badge badge-xs',
						libraryFileAdapter.getCategoryBadgeClass(file.categoryId)
					)}
				>
					{libraryFileAdapter.getCategoryLabel(file.categoryId)}
				</span>
			</button>
		{:else}
			<button
				class="btn px-1 opacity-40 btn-ghost btn-xs hover:opacity-100"
				title="Set category"
				onclick={() => onedittype(file)}
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class="h-3.5 w-3.5"
					fill="none"
					viewBox="0 0 24 24"
					stroke="currentColor"
					stroke-width="2"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						d="M7 7h.01M7 3h5c.512 0 1.024.195 1.414.586l7 7a2 2 0 010 2.828l-7 7a2 2 0 01-2.828 0l-7-7A1.994 1.994 0 013 12V7a4 4 0 014-4z"
					/>
				</svg>
			</button>
		{/if}
	</td>
	<td>
		<span class="uppercase opacity-60">{file.extension}</span>
	</td>
	<td>
		{#if file.mediaType === 'video'}
			{#if tmdbLink}
				<div class="flex items-center gap-1">
					<span class="badge badge-xs badge-info" title={tmdbLabel}>{tmdbLink.serviceId}</span>
					<button
						class="btn px-1 opacity-40 btn-ghost btn-xs hover:text-error hover:opacity-100"
						title="Unlink TMDB"
						onclick={() => onunlinkclick(file)}
					>
						&times;
					</button>
				</div>
			{:else}
				<button
					class="btn px-1 opacity-40 btn-ghost btn-xs hover:opacity-100"
					title="Link TMDB"
					onclick={() => onlinkclick(file)}
				>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="h-3.5 w-3.5"
						fill="none"
						viewBox="0 0 24 24"
						stroke="currentColor"
						stroke-width="2"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1"
						/>
					</svg>
				</button>
			{/if}
		{:else}
			<span class="opacity-30">—</span>
		{/if}
	</td>
	<td>
		{#if file.mediaType === 'video' || file.mediaType === 'audio'}
			{#if editingYoutube}
				<div class="flex items-center gap-1">
					<input
						type="text"
						class="input-bordered input input-xs w-24"
						placeholder="Video ID"
						bind:value={youtubeInput}
						onkeydown={handleYoutubeKeydown}
						onblur={cancelYoutubeEdit}
					/>
				</div>
			{:else if file.links.youtube}
				<div class="flex items-center gap-1">
					<button
						class="btn px-0 btn-ghost btn-xs"
						title="Preview YouTube video"
						onclick={() => onyoutubepreview(file)}
					>
						<span class="badge badge-xs badge-secondary" title={file.links.youtube.serviceId}
							>{file.links.youtube.serviceId}</span
						>
					</button>
					<button
						class="btn px-1 opacity-40 btn-ghost btn-xs hover:text-error hover:opacity-100"
						title="Remove YouTube ID"
						onclick={() => onyoutubeunlink(file)}
					>
						&times;
					</button>
				</div>
			{:else}
				<button
					class="btn px-1 opacity-40 btn-ghost btn-xs hover:opacity-100"
					title="Set YouTube ID"
					onclick={startYoutubeEdit}
				>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="h-3.5 w-3.5"
						fill="none"
						viewBox="0 0 24 24"
						stroke="currentColor"
						stroke-width="2"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z"
						/>
					</svg>
				</button>
			{/if}
		{:else}
			<span class="opacity-30">—</span>
		{/if}
	</td>
	<td>
		{#if file.mediaType === 'audio'}
			{#if file.links.musicbrainz}
				<div class="flex items-center gap-1">
					<span
						class="badge max-w-20 truncate badge-xs badge-accent"
						title={file.links.musicbrainz.serviceId}>{file.links.musicbrainz.serviceId}</span
					>
					<button
						class="btn px-1 opacity-40 btn-ghost btn-xs hover:text-error hover:opacity-100"
						title="Unlink MusicBrainz"
						onclick={() => onmusicbrainzunlink(file)}
					>
						&times;
					</button>
				</div>
			{:else}
				<button
					class="btn px-1 opacity-40 btn-ghost btn-xs hover:opacity-100"
					title="Link MusicBrainz"
					onclick={() => onmusicbrainzlinkclick(file)}
				>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="h-3.5 w-3.5"
						fill="none"
						viewBox="0 0 24 24"
						stroke="currentColor"
						stroke-width="2"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1"
						/>
					</svg>
				</button>
			{/if}
		{:else}
			<span class="opacity-30">—</span>
		{/if}
	</td>
</tr>
