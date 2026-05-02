<script lang="ts">
	import classNames from 'classnames';
	import { blo } from 'blo';
	import { Icon, addonKind, type FirkinKind, type IconName } from 'cloud-ui';
	import type { CloudFirkin } from '$types/firkin.type';
	import { getCachedImageUrl } from '$services/image-cache.service';

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
		'youtube channel': 'delapouite/tv-tower',
		book: 'delapouite/book-cover',
		game: 'delapouite/gamepad'
	};

	let placeholderIcon = $derived<IconName>(
		KIND_ICON[addonKind(firkin.addon) ?? 'movie'] ?? 'delapouite/film-strip'
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

	function formatScore(value: number): string {
		const rounded = Math.round(value * 10) / 10;
		return Number.isInteger(rounded) ? rounded.toFixed(0) : rounded.toFixed(1);
	}

	function formatVotes(count: number): string {
		if (count >= 1_000_000) return `${(count / 1_000_000).toFixed(1)}M votes`;
		if (count >= 1000) return `${(count / 1000).toFixed(1)}k votes`;
		return `${count} vote${count === 1 ? '' : 's'}`;
	}
</script>

<article class={classNames('group card overflow-hidden rounded-md bg-base-200 shadow-sm', classes)}>
	<header
		class="flex items-baseline justify-between gap-3 border-b border-base-content/10 px-4 py-3"
	>
		<h3 class="flex-1 text-center text-base font-semibold [overflow-wrap:anywhere]">
			{firkin.title}
		</h3>
		{#if creatorIdenticon && !coverImage}
			<img
				src={creatorIdenticon}
				alt=""
				class="h-6 w-6 shrink-0 rounded-full"
				title={`Creator: ${creatorAddress}`}
				aria-label={`Creator: ${creatorAddress}`}
			/>
		{/if}
	</header>
	{#if coverImage}
		<figure class="relative overflow-hidden bg-base-300">
			{#if resolvedCoverUrl}
				<img
					src={resolvedCoverUrl}
					alt={firkin.title}
					width={coverImage.width || undefined}
					height={coverImage.height || undefined}
					class="block h-auto w-full"
					loading="lazy"
				/>
			{:else}
				<div
					class="flex aspect-[2/3] w-full items-center justify-center text-base-content/30"
					aria-hidden="true"
				>
					<Icon name={placeholderIcon} size="40%" />
				</div>
			{/if}
			{#if firkin.description}
				<figcaption
					class="pointer-events-none absolute inset-x-0 bottom-0 bg-black/50 px-4 py-3 text-xs [overflow-wrap:anywhere] whitespace-pre-wrap text-white opacity-0 transition-opacity group-hover:opacity-100"
				>
					{firkin.description}
				</figcaption>
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
	{:else if firkin.description}
		<p
			class="border-b border-base-content/10 px-4 py-3 text-xs [overflow-wrap:anywhere] whitespace-pre-wrap text-base-content/80"
		>
			{firkin.description}
		</p>
	{/if}
	{#if reviews.length > 0}
		<div class="flex flex-wrap items-center gap-1.5 border-t border-base-content/10 px-4 py-2">
			{#each reviews as review (review.label)}
				<span
					class="badge gap-1 badge-outline font-mono badge-sm"
					title={review.voteCount !== undefined
						? `${review.label}: ${formatScore(review.score)} / ${formatScore(review.maxScore)} (${formatVotes(review.voteCount)})`
						: `${review.label}: ${formatScore(review.score)} / ${formatScore(review.maxScore)}`}
				>
					<span class="font-semibold">{review.label}</span>
					<span>{formatScore(review.score)} / {formatScore(review.maxScore)}</span>
					{#if review.voteCount !== undefined}
						<span class="text-base-content/60">· {formatVotes(review.voteCount)}</span>
					{/if}
				</span>
			{/each}
		</div>
	{/if}
</article>
