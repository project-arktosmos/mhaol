<script lang="ts">
	import classNames from 'classnames';
	import type { MediaItem } from 'ui-lib/types/media-card.type';
	import { libraryFileAdapter } from 'ui-lib/adapters/classes/library-file.adapter';
	import type { MediaType } from 'ui-lib/types/library.type';
	import type { TorrentState } from 'ui-lib/types/torrent.type';
	import TorrentProgressOverlay from 'ui-lib/components/torrent/TorrentProgressOverlay.svelte';
	import type { Snippet } from 'svelte';

	interface Props {
		item: MediaItem;
		imageUrl?: string | null;
		imageAlt?: string;
		loading?: boolean;
		classes?: string;
		selected?: boolean;
		torrentProgress?: number | null;
		torrentState?: TorrentState | null;
		torrentSpeed?: number | null;
		torrentEta?: number | null;
		href?: string;
		onclick?: () => void;
		children?: Snippet;
	}

	let {
		item,
		imageUrl = null,
		imageAlt = '',
		loading = false,
		classes = '',
		selected = false,
		torrentProgress = null,
		torrentState = null,
		torrentSpeed = null,
		torrentEta = null,
		href,
		onclick,
		children
	}: Props = $props();

	let isDownloading = $derived(
		torrentProgress !== null && torrentState !== null && torrentState !== 'seeding'
	);
</script>

{#snippet cardBody()}
	<figure class="relative h-48 overflow-hidden bg-base-300">
		{#if loading}
			<div class="flex h-full w-full items-center justify-center">
				<span class="loading loading-md loading-spinner"></span>
			</div>
		{:else if imageUrl}
			<img src={imageUrl} alt={imageAlt} class="h-full w-full object-cover" loading="lazy" />
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
						d="M7 4v16M17 4v16M3 8h4m10 0h4M3 12h18M3 16h4m10 0h4M4 20h16a1 1 0 001-1V5a1 1 0 00-1-1H4a1 1 0 00-1 1v14a1 1 0 001 1z"
					/>
				</svg>
			</div>
		{/if}
		{#if isDownloading}
			<TorrentProgressOverlay {torrentProgress} {torrentState} {torrentSpeed} {torrentEta} />
		{/if}
	</figure>
	<div class="card-body gap-1">
		<h3 class="card-title truncate text-sm" title={item.name}>{item.name}</h3>
		<div class="flex flex-wrap gap-1">
			<span
				class={classNames(
					'badge badge-xs',
					libraryFileAdapter.getMediaTypeBadgeClass(item.mediaTypeId as MediaType)
				)}
			>
				{libraryFileAdapter.getMediaTypeLabel(item.mediaTypeId as MediaType)}
			</span>
			{#if item.categoryId}
				<span
					class={classNames(
						'badge badge-xs',
						libraryFileAdapter.getCategoryBadgeClass(item.categoryId)
					)}
				>
					{libraryFileAdapter.getCategoryLabel(item.categoryId)}
				</span>
			{/if}
			<span class="badge badge-ghost badge-xs">{item.extension}</span>
		</div>
		{#if children}
			{@render children()}
		{/if}
	</div>
{/snippet}

{#if href}
	<a
		{href}
		class={classNames(
			'card-compact card bg-base-200 text-inherit no-underline shadow-sm',
			{
				'ring-2 ring-primary': selected,
				'cursor-pointer transition-shadow hover:shadow-md': true
			},
			classes
		)}
	>
		{@render cardBody()}
	</a>
{:else}
	<div
		class={classNames(
			'card-compact card bg-base-200 shadow-sm',
			{
				'ring-2 ring-primary': selected,
				'cursor-pointer transition-shadow hover:shadow-md': !!onclick
			},
			classes
		)}
		{onclick}
		role={onclick ? 'button' : undefined}
		tabindex={onclick ? 0 : undefined}
		onkeydown={onclick
			? (e) => {
					if (e.key === 'Enter' || e.key === ' ') {
						e.preventDefault();
						onclick?.();
					}
				}
			: undefined}
	>
		{@render cardBody()}
	</div>
{/if}
