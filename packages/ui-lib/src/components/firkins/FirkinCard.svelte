<script lang="ts">
	import classNames from 'classnames';
	import { blo } from 'blo';
	import type { CloudFirkin } from 'ui-lib/types/firkin.type';
	import { getCachedImageUrl } from 'ui-lib/services/image-cache.service';

	interface Props {
		firkin: CloudFirkin;
		classes?: string;
	}

	let { firkin, classes = '' }: Props = $props();

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
	let creatorIdenticon = $derived(
		creatorAddress ? blo(creatorAddress as `0x${string}`) : null
	);
</script>

<article class={classNames('group card bg-base-200 shadow-sm', classes)}>
	{#if !coverImage}
		<header
			class="flex items-baseline justify-between gap-3 border-b border-base-content/10 px-4 py-3"
		>
			<h3 class="flex-1 text-center text-base font-semibold [overflow-wrap:anywhere]">
				{firkin.title}
			</h3>
			{#if creatorIdenticon}
				<img
					src={creatorIdenticon}
					alt=""
					class="h-6 w-6 shrink-0 rounded-full"
					title={`Creator: ${creatorAddress}`}
					aria-label={`Creator: ${creatorAddress}`}
				/>
			{/if}
		</header>
	{/if}
	{#if coverImage}
		<figure class="relative overflow-hidden bg-base-300">
			<h3
				class="absolute inset-x-0 top-0 z-10 bg-black/60 px-4 py-2 text-center text-base font-semibold [overflow-wrap:anywhere] text-white"
			>
				{firkin.title}
			</h3>
			{#if resolvedCoverUrl}
				<img
					src={resolvedCoverUrl}
					alt={firkin.title}
					width={coverImage.width || undefined}
					height={coverImage.height || undefined}
					class="block h-auto w-full"
					loading="lazy"
				/>
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
</article>
