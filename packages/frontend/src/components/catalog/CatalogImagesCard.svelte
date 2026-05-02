<script lang="ts">
	import { cachedImageUrl } from '$lib/image-cache';
	import type { ImageMeta } from '$lib/firkins.service';

	interface Props {
		images: ImageMeta[];
	}
	let { images }: Props = $props();

	function formatBytes(bytes: number): string {
		if (!Number.isFinite(bytes) || bytes <= 0) return '—';
		const units = ['B', 'KB', 'MB', 'GB', 'TB'];
		let value = bytes;
		let unit = 0;
		while (value >= 1024 && unit < units.length - 1) {
			value /= 1024;
			unit++;
		}
		return `${value.toFixed(value >= 10 || unit === 0 ? 0 : 1)} ${units[unit]}`;
	}
</script>

{#if images.length > 0}
	<div class="card border border-base-content/10 bg-base-200 p-4">
		<h2 class="mb-2 text-sm font-semibold text-base-content/70 uppercase">
			Images ({images.length})
		</h2>
		<div class="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4">
			{#each images as image, i (i)}
				<figure
					class="flex flex-col gap-1 overflow-hidden rounded-box border border-base-content/10 bg-base-300"
				>
					<img
						src={cachedImageUrl(image.url)}
						alt={`Image ${i + 1}`}
						class="block h-auto w-full"
						loading="lazy"
					/>
					<figcaption class="px-2 py-1 text-[10px] text-base-content/70">
						{image.width || '?'}×{image.height || '?'}
						{#if image.fileSize}· {formatBytes(image.fileSize)}{/if}
						{#if image.mimeType}· {image.mimeType}{/if}
					</figcaption>
				</figure>
			{/each}
		</div>
	</div>
{/if}
