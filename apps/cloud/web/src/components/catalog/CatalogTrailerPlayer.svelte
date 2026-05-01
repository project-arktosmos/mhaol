<script lang="ts">
	import classNames from 'classnames';
	import { resolveYouTubeStreamUrl } from '$lib/youtube-match.service';

	interface Props {
		posterUrl: string | null;
		youtubeUrl: string | null;
		title: string;
	}

	let { posterUrl, youtubeUrl, title }: Props = $props();

	let videoElement = $state<HTMLVideoElement | null>(null);
	let streamUrl = $state<string | null>(null);
	let streamMime = $state<string | null>(null);
	let started = $state(false);
	let starting = $state(false);
	let error = $state<string | null>(null);
	let resolvedYoutubeUrl: string | null = null;

	$effect(() => {
		if (!youtubeUrl) return;
		if (resolvedYoutubeUrl === youtubeUrl) return;
		resolvedYoutubeUrl = youtubeUrl;
		streamUrl = null;
		streamMime = null;
		started = false;
		error = null;
		void resolveStream(youtubeUrl);
	});

	async function resolveStream(url: string): Promise<void> {
		const resolved = await resolveYouTubeStreamUrl(url);
		if (resolvedYoutubeUrl !== url) return;
		if (!resolved) {
			error = 'No playable trailer format';
			return;
		}
		streamUrl = resolved.url;
		streamMime = resolved.mimeType;
	}

	function handleStart(): void {
		if (!videoElement || !streamUrl || starting || started) return;
		starting = true;
		error = null;
		videoElement
			.play()
			.then(() => {
				started = true;
				starting = false;
			})
			.catch((err: Error) => {
				error = `Playback failed: ${err.message}`;
				starting = false;
			});
	}
</script>

<div class="relative aspect-video w-full overflow-hidden rounded-md bg-black">
	{#if streamUrl}
		<video
			bind:this={videoElement}
			src={streamUrl}
			class="absolute inset-0 h-full w-full bg-black"
			playsinline
			controls={started}
			preload="auto"
			aria-label={title}
		>
			{#if streamMime}
				<source src={streamUrl} type={streamMime} />
			{/if}
		</video>
	{/if}

	{#if posterUrl}
		<div
			class={classNames(
				'pointer-events-none absolute inset-0 bg-cover bg-center transition-opacity duration-500',
				started ? 'opacity-0' : 'opacity-100'
			)}
			style:background-image={`url(${posterUrl})`}
			aria-hidden="true"
		></div>
	{/if}

	{#if !started && (streamUrl || posterUrl)}
		<button
			type="button"
			class="absolute inset-0 z-20 flex items-center justify-center bg-black/30 transition-colors hover:bg-black/40 disabled:cursor-wait"
			aria-label="Play trailer"
			onclick={handleStart}
			disabled={!streamUrl || starting}
		>
			<span
				class={classNames(
					'flex h-20 w-20 items-center justify-center rounded-full bg-primary text-primary-content shadow-lg transition-transform',
					streamUrl && !starting ? 'hover:scale-110' : 'opacity-70'
				)}
			>
				{#if starting || !streamUrl}
					<span class="loading loading-md loading-spinner"></span>
				{:else}
					<svg
						xmlns="http://www.w3.org/2000/svg"
						viewBox="0 0 24 24"
						fill="currentColor"
						class="h-10 w-10 translate-x-0.5"
						aria-hidden="true"
					>
						<polygon points="6 4 20 12 6 20 6 4" />
					</svg>
				{/if}
			</span>
		</button>
	{/if}

	{#if error}
		<div
			class="absolute inset-x-2 bottom-2 z-30 rounded bg-error/90 px-2 py-1 text-xs text-error-content"
		>
			{error}
		</div>
	{/if}
</div>
