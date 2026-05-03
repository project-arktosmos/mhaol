<script lang="ts">
	import { base } from '$app/paths';
	import { Icon } from 'cloud-ui';
</script>

<section
	class="relative overflow-hidden border-b border-base-300 bg-gradient-to-b from-base-100 to-base-200"
>
	<div class="mx-auto flex w-full max-w-6xl flex-col gap-10 px-4 py-20 md:flex-row md:items-center">
		<div class="flex-1">
			<h1 class="text-4xl leading-tight font-bold tracking-tight md:text-6xl">
				Your media.<br />
				Your hardware.<br />
				<span class="text-primary">Your rules.</span>
			</h1>
			<p class="mt-5 max-w-xl text-base text-base-content/80 md:text-lg">
				A Netflix-grade experience, ad-free and subscription-free. Mhaol pulls movies and TV shows
				through its built-in torrent client, downloads music and videos straight from YouTube with
				yt-dlp (no embedded iframes), and streams the result to any device on your network. Stream
				what you want. Keep what you watch. Owe no one a thing.
			</p>
			<div class="mt-7 flex flex-wrap items-center gap-3">
				<a href="{base}/#install" class="btn gap-2 btn-lg btn-primary">
					<Icon name="delapouite/cloud-download" size={20} />
					Get Mhaol
				</a>
			</div>
		</div>

		<div class="flex-1">
			<div
				class="overflow-hidden rounded-box border border-base-300 bg-base-100 shadow-xl ring-1 ring-base-content/5"
			>
				<div class="flex items-center gap-2 border-b border-base-300 bg-base-200 px-3 py-2">
					<span class="h-3 w-3 rounded-full bg-error/70"></span>
					<span class="h-3 w-3 rounded-full bg-warning/70"></span>
					<span class="h-3 w-3 rounded-full bg-success/70"></span>
					<span class="ml-3 font-mono text-xs text-base-content/60">Mhaol Cloud — Catalog</span>
				</div>
				<img
					src="{base}/mhaol-catalog.png"
					alt="Mhaol Cloud catalog showing rows of Movies, TV Shows, Music, and YouTube libraries"
					class="block h-auto w-full"
					loading="eager"
					decoding="async"
				/>
			</div>
		</div>
	</div>
</section>
