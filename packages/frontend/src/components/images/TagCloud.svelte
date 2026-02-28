<script lang="ts">
	import classNames from 'classnames';
	import type { ImageItem } from '$types/image-tagger.type';

	interface Props {
		images: ImageItem[];
		activeFilter: string;
		onselect: (tag: string) => void;
	}

	let { images, activeFilter, onselect }: Props = $props();

	let tagCounts = $derived.by(() => {
		const counts = new Map<string, number>();
		for (const img of images) {
			for (const t of img.tags) {
				counts.set(t.tag, (counts.get(t.tag) ?? 0) + 1);
			}
		}
		return [...counts.entries()]
			.sort((a, b) => b[1] - a[1])
			.map(([tag, count]) => ({ tag, count }));
	});

	let maxCount = $derived(tagCounts.length > 0 ? tagCounts[0].count : 1);
</script>

{#if tagCounts.length > 0}
	<div class="flex flex-wrap gap-2">
		{#each tagCounts as { tag, count } (tag)}
			{@const ratio = count / maxCount}
			{@const isActive = activeFilter.toLowerCase() === tag.toLowerCase()}
			<button
				class={classNames('badge cursor-pointer transition-opacity', {
					'badge-primary': isActive,
					'badge-ghost': !isActive && ratio < 0.3,
					'badge-info badge-outline': !isActive && ratio >= 0.3 && ratio < 0.7,
					'badge-info': !isActive && ratio >= 0.7,
					'badge-sm': ratio < 0.5,
					'badge-md': ratio >= 0.5
				})}
				onclick={() => onselect(isActive ? '' : tag)}
				title={`${count} image${count === 1 ? '' : 's'}`}
			>
				{tag}
				<span class="opacity-60 ml-0.5">{count}</span>
			</button>
		{/each}
	</div>
{/if}
