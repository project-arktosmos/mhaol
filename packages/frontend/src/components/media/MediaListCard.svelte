<script lang="ts">
	import classNames from 'classnames';
	import type { MediaList } from '$types/media-list.type';
	import { libraryFileAdapter } from '$adapters/classes/library-file.adapter';
	import type { MediaType } from '$types/library.type';

	interface Props {
		list: MediaList;
		selected?: boolean;
		onselect?: (list: MediaList) => void;
	}

	let { list, selected = false, onselect }: Props = $props();

	let kindLabel = $derived(list.mediaType === 'video' ? 'TV Show' : 'Album');
</script>

<div
	class={classNames(
		'card-compact card cursor-pointer bg-base-200 shadow-sm transition-shadow hover:shadow-md',
		{
			'ring-2 ring-primary': selected
		}
	)}
	onclick={() => onselect?.(list)}
	role="button"
	tabindex={0}
	onkeydown={(e) => {
		if (e.key === 'Enter' || e.key === ' ') {
			e.preventDefault();
			onselect?.(list);
		}
	}}
>
	<figure class="relative h-48 overflow-hidden bg-base-300">
		{#if list.coverImage}
			<img
				src={list.coverImage}
				alt={list.title}
				class="h-full w-full object-cover"
				loading="lazy"
			/>
		{:else}
			<div class="flex h-full w-full items-center justify-center text-base-content/20">
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class="h-12 w-12"
					fill="none"
					viewBox="0 0 24 24"
					stroke="currentColor"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="1"
						d="M4 6h16M4 10h16M4 14h16M4 18h16"
					/>
				</svg>
			</div>
		{/if}
	</figure>
	<div class="card-body gap-1">
		<h3 class="card-title truncate text-sm" title={list.title}>{list.title}</h3>
		<div class="flex flex-wrap gap-1">
			<span
				class={classNames(
					'badge badge-xs',
					libraryFileAdapter.getMediaTypeBadgeClass(list.mediaType as MediaType)
				)}
			>
				{kindLabel}
			</span>
			<span class="badge badge-ghost badge-xs">{list.itemCount} items</span>
		</div>
	</div>
</div>
