<script lang="ts">
	import type { FirkinImage } from '../../types/firkin.js';

	interface Props {
		images: FirkinImage[];
		/**
		 * Optional URL transformer — the cloud WebUI swaps remote URLs for
		 * its `/api/image-cache?url=` proxy. The player passes through
		 * untouched. Default: identity.
		 */
		resolveUrl?: (url: string) => string;
	}
	let { images, resolveUrl = (u) => u }: Props = $props();

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
	<div class="flex flex-col gap-2">
		<h2 class="text-sm font-semibold text-base-content/70 uppercase">
			Images ({images.length})
		</h2>
		<div class="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4">
			{#each images as image, i (i)}
				<figure
					class="flex flex-col gap-1 overflow-hidden rounded-box border border-base-content/10 bg-base-300"
				>
					<img
						src={resolveUrl(image.url)}
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
