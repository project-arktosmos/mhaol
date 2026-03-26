<script lang="ts">
	import classNames from 'classnames';
	import type { CatalogCardData } from 'ui-lib/types/catalog.type';

	interface Props {
		card: CatalogCardData;
		onclick?: () => void;
	}

	let { card, onclick }: Props = $props();

	let aspectClass = $derived(
		card.aspectRatio === 'poster'
			? 'aspect-[2/3]'
			: card.aspectRatio === 'square'
				? 'aspect-square'
				: 'aspect-video'
	);

	let containerClasses = $derived(
		classNames(
			'card card-compact bg-base-200 shadow-sm cursor-pointer transition-all hover:shadow-md',
			{
				'ring-2 ring-primary': card.selected,
				'ring-2 ring-success': card.fetched && !card.selected,
				'opacity-40': card.dimmed || card.torrentState === 'paused'
			}
		)
	);

	let ratingColor = $derived(
		card.rating && card.rating >= 7
			? 'text-success'
			: card.rating && card.rating >= 5
				? 'text-warning'
				: 'text-error'
	);

	let dlBadge = $derived.by(() => {
		if (!card.torrentState) return null;
		switch (card.torrentState) {
			case 'downloading':
				return {
					label:
						card.torrentProgress != null
							? `${Math.round(card.torrentProgress * 100)}%`
							: 'Downloading',
					cls: 'badge-primary'
				};
			case 'seeding':
				return { label: 'Seeding', cls: 'badge-success' };
			case 'paused':
				return { label: 'Paused', cls: 'badge-warning' };
			case 'error':
				return { label: 'Error', cls: 'badge-error' };
			case 'initializing':
			case 'checking':
				return { label: card.torrentState, cls: 'badge-info' };
			default:
				return null;
		}
	});
</script>

<div
	class={containerClasses}
	role="button"
	tabindex="0"
	{onclick}
	onkeydown={(e) => e.key === 'Enter' && onclick?.()}
>
	<figure class={classNames('relative overflow-hidden', aspectClass)}>
		{#if card.imageUrl}
			<img
				src={card.imageUrl}
				alt={card.title}
				class="h-full w-full object-cover"
				loading="lazy"
			/>
		{:else}
			<div
				class="flex h-full w-full items-center justify-center bg-base-300 text-base-content/20"
			>
				<svg class="h-12 w-12" fill="currentColor" viewBox="0 0 24 24">
					<path
						d="M21 3H3c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h18c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm0 16H3V5h18v14z"
					/>
				</svg>
			</div>
		{/if}
		{#if card.torrentProgress != null && card.torrentProgress > 0 && card.torrentProgress < 1}
			<div class="absolute right-0 bottom-0 left-0 h-1 bg-base-300">
				<div
					class="h-full bg-primary transition-all"
					style="width: {card.torrentProgress * 100}%"
				></div>
			</div>
		{/if}
		{#if card.loading}
			<div class="absolute right-1 top-1">
				<span class="loading loading-xs loading-spinner"></span>
			</div>
		{:else if card.favorited || card.pinned}
			<div class="absolute top-1 right-1 flex gap-0.5">
				{#if card.favorited}
					<span class="badge badge-xs badge-error">♥</span>
				{/if}
				{#if card.pinned}
					<span class="badge badge-xs badge-info">📌</span>
				{/if}
			</div>
		{/if}
		{#if dlBadge}
			<div class="absolute bottom-1 left-1">
				<span class={classNames('badge badge-xs', dlBadge.cls)}>{dlBadge.label}</span>
			</div>
		{:else if card.fetched}
			<div class="absolute bottom-1 left-1">
				<span class="badge badge-xs badge-success">fetched</span>
			</div>
		{/if}
	</figure>
	<div class="card-body gap-0.5 p-2">
		<h3 class="truncate text-sm font-medium">{card.title}</h3>
		{#if card.fetchCacheSummary}
			<p class="truncate text-xs text-success/70" title={card.fetchCacheSummary}>
				{card.fetchCacheSummary}
			</p>
		{:else if card.subtitle}
			<p class="truncate text-xs opacity-60">{card.subtitle}</p>
		{/if}
		<div class="mt-0.5 flex flex-wrap items-center gap-1">
			{#if card.year}
				<span class="text-xs opacity-50">{card.year}</span>
			{/if}
			{#if card.rating}
				<span class={classNames('text-xs font-semibold', ratingColor)}>
					{card.rating.toFixed(1)}
				</span>
			{/if}
			{#each card.badges.slice(0, 2) as badge}
				<span class={classNames('badge badge-xs', `badge-${badge.variant}`)}>{badge.label}</span>
			{/each}
		</div>
	</div>
</div>
