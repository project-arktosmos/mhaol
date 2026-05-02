<script lang="ts">
	import classNames from 'classnames';
	import { Icon, addonKind, type FirkinKind, type IconName } from 'cloud-ui';
	import type { CloudFirkin } from '$types/firkin.type';
	import { getCachedImageUrl } from '$services/image-cache.service';
	import { firkinTooltipService } from '$services/firkins/firkin-tooltip.svelte';
	import { hashColor } from '$utils/string/hash-color';

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
	let albumBgColor = $derived(
		isAlbum ? `#${hashColor(`${firkin.title}::${artistNames}`)}` : null
	);

	// Musicbrainz firkins are created with description = "credits · primary_type"
	// (or just one of the two). Strip the credits prefix to surface the type.
	let albumType = $derived.by(() => {
		if (!isAlbum) return '';
		const desc = (firkin.description ?? '').trim();
		if (!desc) return '';
		if (artistNames && desc.startsWith(artistNames)) {
			const rest = desc.slice(artistNames.length).trim();
			return rest.startsWith('·') ? rest.slice(1).trim() : '';
		}
		return desc;
	});

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

	// Mirrors the catalog detail page's IPFS-stream gating: an `ipfs`-typed
	// FileEntry whose title ends in a video container we can feed to the
	// hlssink2 pipeline. For musicbrainz albums the gating is broader —
	// any `ipfs`-typed entry counts, because the per-track download flow
	// (POST /api/firkins/:id/download-album) only ever mints audio files
	// and a partially-downloaded album is still meaningfully "in cloud".
	const VIDEO_EXTS = new Set([
		'.mkv',
		'.mp4',
		'.m4v',
		'.mov',
		'.webm',
		'.avi',
		'.ts',
		'.m2ts',
		'.mpg',
		'.mpeg',
		'.ogv',
		'.wmv',
		'.flv'
	]);
	let hasIpfsPlayable = $derived.by(() => {
		if (firkin.addon === 'musicbrainz') {
			return firkin.files.some((f) => f.type === 'ipfs');
		}
		return firkin.files.some((f) => {
			if (f.type !== 'ipfs') return false;
			const title = (f.title ?? '').toLowerCase();
			const dot = title.lastIndexOf('.');
			if (dot < 0) return false;
			return VIDEO_EXTS.has(title.slice(dot));
		});
	});

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
		class={classNames('relative overflow-hidden', {
			'aspect-square w-full': isAlbum,
			'bg-base-300': !isAlbum
		})}
		style={albumBgColor ? `background-color: ${albumBgColor};` : undefined}
	>
		{#if coverImage && resolvedCoverUrl}
			<img
				src={resolvedCoverUrl}
				alt={isAlbum ? '' : firkin.title}
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
		{#if hasIpfsPlayable}
			<div
				class="absolute right-2 bottom-2 z-10 flex h-7 w-7 items-center justify-center rounded-full bg-black/60 text-white ring-1 ring-white/20"
				title="Available via IPFS Stream"
				aria-label="Available via IPFS Stream"
			>
				<Icon name="lorc/fluffy-cloud" size={16} />
			</div>
		{/if}
	</figure>
	{#if isAlbum}
		<div class="px-3 py-2">
			<div class="truncate text-sm font-semibold" title={firkin.title}>{firkin.title}</div>
			{#if artistNames || albumType}
				<div
					class="truncate text-xs text-base-content/70"
					title={[artistNames, albumType].filter(Boolean).join(' · ')}
				>
					{artistNames}{#if artistNames && albumType}<span class="mx-1 text-base-content/40">·</span
						>{/if}{#if albumType}<span class="text-base-content/50">{albumType}</span>{/if}
				</div>
			{/if}
		</div>
	{/if}
</article>
