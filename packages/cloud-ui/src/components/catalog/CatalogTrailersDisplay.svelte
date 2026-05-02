<script lang="ts">
	import type { FirkinTrailer } from '../../types/firkin.js';

	interface Props {
		trailers: FirkinTrailer[];
	}
	let { trailers }: Props = $props();

	let playingKey = $state<string | null>(null);

	function youTubeEmbedUrl(url: string): string | null {
		try {
			const u = new URL(url);
			const host = u.hostname.toLowerCase();
			let id: string | null = null;
			if (host === 'youtu.be') {
				id = u.pathname.replace(/^\//, '').split('/')[0] ?? null;
			} else if (host.endsWith('youtube.com')) {
				if (u.pathname === '/watch') id = u.searchParams.get('v');
				else if (u.pathname.startsWith('/embed/'))
					id = u.pathname.replace('/embed/', '').split('/')[0] ?? null;
				else if (u.pathname.startsWith('/shorts/'))
					id = u.pathname.replace('/shorts/', '').split('/')[0] ?? null;
			}
			if (!id) return null;
			return `https://www.youtube.com/embed/${encodeURIComponent(id)}?autoplay=1`;
		} catch {
			return null;
		}
	}
</script>

{#if trailers.length > 0}
	<div class="card border border-base-content/10 bg-base-200 p-4">
		<h2 class="mb-2 text-sm font-semibold text-base-content/70 uppercase">
			Trailers ({trailers.length})
		</h2>
		<ul class="flex flex-col gap-2">
			{#each trailers as trailer, i (trailer.youtubeUrl + ':' + i)}
				{@const key = trailer.youtubeUrl + ':' + i}
				{@const embed = youTubeEmbedUrl(trailer.youtubeUrl)}
				<li class="flex flex-col gap-2">
					<div class="flex items-center justify-between gap-2">
						<div class="flex min-w-0 flex-col">
							{#if trailer.label}
								<span class="text-sm font-semibold">{trailer.label}</span>
							{/if}
							<a
								href={trailer.youtubeUrl}
								target="_blank"
								rel="noreferrer"
								class="link truncate text-xs"
							>
								{trailer.youtubeUrl}
							</a>
						</div>
						{#if embed}
							<button
								type="button"
								class="btn btn-xs"
								onclick={() => (playingKey = playingKey === key ? null : key)}
							>
								{playingKey === key ? 'Hide' : 'Play'}
							</button>
						{/if}
					</div>
					{#if embed && playingKey === key}
						<div class="aspect-video w-full overflow-hidden rounded">
							<iframe
								class="h-full w-full"
								src={embed}
								title={trailer.label ?? 'Trailer'}
								allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share"
								allowfullscreen
							></iframe>
						</div>
					{/if}
				</li>
			{/each}
		</ul>
	</div>
{/if}
