<script lang="ts">
	import classNames from 'classnames';
	import type { MediaItem } from '$types/media-card.type';
	import { libraryFileAdapter } from '$adapters/classes/library-file.adapter';
	import type { MediaType } from '$types/library.type';
	import type { Snippet } from 'svelte';

	interface Props {
		item: MediaItem;
		imageUrl?: string | null;
		imageAlt?: string;
		loading?: boolean;
		classes?: string;
		children?: Snippet;
		actions?: Snippet;
	}

	let { item, imageUrl = null, imageAlt = '', loading = false, classes = '', children, actions }: Props = $props();
</script>

<div class={classNames('card card-compact bg-base-200 shadow-sm', classes)}>
	<figure class="relative h-48 overflow-hidden bg-base-300">
		{#if loading}
			<div class="flex h-full w-full items-center justify-center">
				<span class="loading loading-spinner loading-md"></span>
			</div>
		{:else if imageUrl}
			<img
				src={imageUrl}
				alt={imageAlt}
				class="h-full w-full object-cover"
				loading="lazy"
			/>
		{:else}
			<div class="flex h-full w-full items-center justify-center text-base-content/20">
				<svg xmlns="http://www.w3.org/2000/svg" class="h-12 w-12" fill="none" viewBox="0 0 24 24" stroke="currentColor">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="1" d="M7 4v16M17 4v16M3 8h4m10 0h4M3 12h18M3 16h4m10 0h4M4 20h16a1 1 0 001-1V5a1 1 0 00-1-1H4a1 1 0 00-1 1v14a1 1 0 001 1z" />
				</svg>
			</div>
		{/if}
	</figure>
	<div class="card-body gap-1">
		<h3 class="card-title truncate text-sm" title={item.name}>{item.name}</h3>
		<div class="flex flex-wrap gap-1">
			<span class={classNames('badge badge-xs', libraryFileAdapter.getMediaTypeBadgeClass(item.mediaTypeId as MediaType))}>
				{libraryFileAdapter.getMediaTypeLabel(item.mediaTypeId as MediaType)}
			</span>
			{#if item.categoryId}
				<span class={classNames('badge badge-xs', libraryFileAdapter.getCategoryBadgeClass(item.categoryId))}>
					{libraryFileAdapter.getCategoryLabel(item.categoryId)}
				</span>
			{/if}
			<span class="badge badge-ghost badge-xs">{item.extension}</span>
		</div>
		{#if children}
			{@render children()}
		{/if}
		{#if actions}
			<div class="card-actions mt-1 justify-end">
				{@render actions()}
			</div>
		{/if}
	</div>
</div>
