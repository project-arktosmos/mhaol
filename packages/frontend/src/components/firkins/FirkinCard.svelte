<script lang="ts">
	import classNames from 'classnames';
	import { blo } from 'blo';
	import { Icon, addonKind, type FirkinKind, type IconName } from 'cloud-ui';
	import type { CloudFirkin } from '$types/firkin.type';
	import { getCachedImageUrl } from '$services/image-cache.service';
	import { firkinTooltipService } from '$services/firkins/firkin-tooltip.svelte';

	interface Props {
		firkin: CloudFirkin;
		classes?: string;
	}

	let { firkin, classes = '' }: Props = $props();

	const KIND_ICON: Record<FirkinKind, IconName> = {
		movie: 'delapouite/film-strip',
		'tv show': 'delapouite/tv',
		album: 'delapouite/compact-disc',
		'youtube video': 'delapouite/video-camera',
		book: 'delapouite/book-cover',
		game: 'delapouite/gamepad'
	};

	let placeholderIcon = $derived<IconName>(
		KIND_ICON[addonKind(firkin.addon) ?? 'movie'] ?? 'delapouite/film-strip'
	);

	let kind = $derived(addonKind(firkin.addon));
	let isAlbum = $derived(kind === 'album');
	let artistNames = $derived(
		(firkin.artists ?? [])
			.map((a) => a.name)
			.filter((n) => n && n.length > 0)
			.join(', ')
	);

	let coverImage = $derived(firkin.images?.[0] ?? null);
	let resolvedCoverUrl = $state<string | null>(null);

	$effect(() => {
		const url = coverImage?.url;
		if (!url) {
			resolvedCoverUrl = null;
			return;
		}
		let cancelled = false;
		getCachedImageUrl(url).then((u) => {
			if (!cancelled) resolvedCoverUrl = u;
		});
		return () => {
			cancelled = true;
		};
	});

	let creatorAddress = $derived(firkin.creator ?? '');
	let creatorIdenticon = $derived(creatorAddress ? blo(creatorAddress as `0x${string}`) : null);

	let reviews = $derived(firkin.reviews ?? []);

	let tooltipImageUrl = $derived(firkin.images?.[firkin.images.length - 1]?.url ?? null);

	function handlePointerEnter(event: PointerEvent) {
		firkinTooltipService.show(
			{
				title: firkin.title,
				description: firkin.description ?? null,
				imageUrl: tooltipImageUrl,
				reviews
			},
			event.clientX,
			event.clientY
		);
	}

	function handlePointerMove(event: PointerEvent) {
		firkinTooltipService.move(event.clientX, event.clientY);
	}

	function handlePointerLeave() {
		firkinTooltipService.hide();
	}

	// Hide the hover tooltip the instant a firkin card is clicked — the
	// click typically navigates away to a detail page and otherwise the
	// tooltip would linger over the new view until the next pointer move.
	function handleClick() {
		firkinTooltipService.hide();
	}
</script>

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_noninteractive_element_interactions -->
<article
	class={classNames(
		'card w-full overflow-hidden rounded-md bg-base-200 shadow-sm',
		classes
	)}
	onpointerenter={handlePointerEnter}
	onpointermove={handlePointerMove}
	onpointerleave={handlePointerLeave}
	onclick={handleClick}
>
	<figure
		class={classNames('relative overflow-hidden bg-base-300', {
			'aspect-square w-full': isAlbum
		})}
	>
		{#if coverImage && resolvedCoverUrl}
			<img
				src={resolvedCoverUrl}
				alt={firkin.title}
				width={coverImage.width || undefined}
				height={coverImage.height || undefined}
				class={classNames('block w-full', isAlbum ? 'h-full object-cover' : 'h-auto')}
				loading="lazy"
			/>
		{:else}
			<div
				class={classNames(
					'flex w-full items-center justify-center text-base-content/30',
					isAlbum ? 'h-full' : 'aspect-[2/3]'
				)}
				aria-hidden="true"
			>
				<Icon name={placeholderIcon} size="40%" />
			</div>
		{/if}
		{#if creatorIdenticon}
			<img
				src={creatorIdenticon}
				alt=""
				class="absolute right-2 bottom-2 z-10 h-8 w-8 rounded-full ring-2 ring-black/40"
				title={`Creator: ${creatorAddress}`}
				aria-label={`Creator: ${creatorAddress}`}
			/>
		{/if}
	</figure>
	{#if isAlbum}
		<div class="px-3 py-2">
			<div class="truncate text-sm font-semibold" title={firkin.title}>{firkin.title}</div>
			{#if artistNames}
				<div class="truncate text-xs text-base-content/70" title={artistNames}>
					{artistNames}
				</div>
			{/if}
		</div>
	{/if}
</article>
