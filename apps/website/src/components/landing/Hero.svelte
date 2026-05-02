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
				<a href="{base}/#why" class="btn btn-ghost btn-lg">Why Mhaol</a>
			</div>
		</div>

		<div class="flex-1">
			<div class="card border border-base-300 bg-base-100 shadow-xl">
				<div class="card-body p-5">
					<div class="flex items-center gap-2 border-b border-base-300 pb-3">
						<span class="h-3 w-3 rounded-full bg-error/70"></span>
						<span class="h-3 w-3 rounded-full bg-warning/70"></span>
						<span class="h-3 w-3 rounded-full bg-success/70"></span>
						<span class="ml-3 font-mono text-xs text-base-content/60">My library</span>
					</div>

					<div class="mt-3 grid grid-cols-4 gap-3 text-center">
						<div class="rounded-md bg-base-200 p-3">
							<div class="text-lg font-bold">142</div>
							<div class="text-[0.65rem] tracking-wide text-base-content/60 uppercase">Movies</div>
						</div>
						<div class="rounded-md bg-base-200 p-3">
							<div class="text-lg font-bold">38</div>
							<div class="text-[0.65rem] tracking-wide text-base-content/60 uppercase">Shows</div>
						</div>
						<div class="rounded-md bg-base-200 p-3">
							<div class="text-lg font-bold">412</div>
							<div class="text-[0.65rem] tracking-wide text-base-content/60 uppercase">Albums</div>
						</div>
						<div class="rounded-md bg-base-200 p-3">
							<div class="text-lg font-bold">86</div>
							<div class="text-[0.65rem] tracking-wide text-base-content/60 uppercase">Videos</div>
						</div>
					</div>

					<div class="mt-3 rounded-md border border-base-300 p-3">
						<div class="flex items-center gap-2 text-xs text-base-content/60">
							<Icon name="delapouite/film-strip" size={14} class="text-primary" />
							Now playing
						</div>
						<div class="mt-1 flex items-center justify-between text-sm">
							<span class="font-medium">Dune: Part Two</span>
							<span class="font-mono text-xs text-base-content/60">1:24:03 / 2:46:00</span>
						</div>
						<div class="mt-2 h-1.5 w-full overflow-hidden rounded-full bg-base-200">
							<div class="h-full w-[51%] bg-primary"></div>
						</div>
					</div>

					<div class="mt-3 flex items-center justify-between text-xs text-base-content/60">
						<span class="inline-flex items-center gap-1.5">
							<Icon name="delapouite/wifi-router" size={14} />
							3 devices on your network
						</span>
						<span class="inline-flex items-center gap-1.5">
							<Icon name="delapouite/cube" size={14} />
							Library pinned
						</span>
					</div>
				</div>
			</div>
		</div>
	</div>
</section>
