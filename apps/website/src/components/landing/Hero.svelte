<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import { base } from '$app/paths';
	import { Icon } from 'cloud-ui';

	const slides = [
		{ src: 'mhaol-catalog.png', label: 'Catalog' },
		{ src: 'mhaol-catalog-posters.png', label: 'Catalog · Poster view' },
		{ src: 'mhaol-movies.png', label: 'Movies' },
		{ src: 'mhaol-album-detail.png', label: 'She Wolf' }
	];

	let active = $state(0);
	let timer: ReturnType<typeof setInterval> | null = null;

	onMount(() => {
		timer = setInterval(() => {
			active = (active + 1) % slides.length;
		}, 3000);
	});

	onDestroy(() => {
		if (timer !== null) clearInterval(timer);
	});
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
					<span class="ml-3 font-mono text-xs text-base-content/60">
						Mhaol Cloud — {slides[active].label}
					</span>
				</div>
				<div class="relative aspect-[5/3] w-full bg-base-200">
					{#each slides as slide, i (slide.src)}
						<img
							src="{base}/{slide.src}"
							alt={`Mhaol Cloud screenshot — ${slide.label}`}
							class={[
								'absolute inset-0 h-full w-full object-contain transition-opacity duration-700 ease-in-out',
								i === active ? 'opacity-100' : 'opacity-0'
							]}
							loading={i === 0 ? 'eager' : 'lazy'}
							decoding="async"
							aria-hidden={i === active ? undefined : 'true'}
						/>
					{/each}
				</div>
				<div
					class="flex items-center justify-center gap-1.5 border-t border-base-300 bg-base-200 py-2"
				>
					{#each slides as _, i (i)}
						<span
							class={[
								'h-1.5 w-1.5 rounded-full transition-colors',
								i === active ? 'bg-primary' : 'bg-base-content/20'
							]}
							aria-hidden="true"
						></span>
					{/each}
				</div>
			</div>
		</div>
	</div>
</section>
