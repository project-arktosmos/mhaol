<script lang="ts">
	import classNames from 'classnames';
	import type { LibraryFile } from '$types/library.type';
	import { libraryFileAdapter } from '$adapters/classes/library-file.adapter';

	interface Props {
		file: LibraryFile;
		onlinkclick: (file: LibraryFile, type: 'movie' | 'tv') => void;
		onunlinkclick: (file: LibraryFile) => void;
		onedittype: (file: LibraryFile) => void;
	}

	let { file, onlinkclick, onunlinkclick, onedittype }: Props = $props();

	let tmdbLink = $derived(file.links.tmdb ?? null);
	let tmdbLabel = $derived.by(() => {
		if (!tmdbLink) return '';
		let label = `TMDB ${tmdbLink.serviceId}`;
		if (tmdbLink.seasonNumber != null) label += ` S${tmdbLink.seasonNumber}`;
		if (tmdbLink.episodeNumber != null) label += `E${tmdbLink.episodeNumber}`;
		return label;
	});
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
				<div class="flex gap-0.5">
					<button
						class="btn px-1 opacity-40 btn-ghost btn-xs hover:opacity-100"
						title="Link Movie"
						onclick={() => onlinkclick(file, 'movie')}
					>
						M
					</button>
					<button
						class="btn px-1 opacity-40 btn-ghost btn-xs hover:opacity-100"
						title="Link TV Show"
						onclick={() => onlinkclick(file, 'tv')}
					>
						TV
					</button>
				</div>
			{/if}
		{:else}
			<span class="opacity-30">—</span>
		{/if}
	</td>
</tr>
