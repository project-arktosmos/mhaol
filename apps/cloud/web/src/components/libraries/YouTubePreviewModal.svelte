<script lang="ts">
	import type { LibraryFile } from '$types/library.type';

	interface Props {
		file: LibraryFile;
		videoId: string;
		onclose: () => void;
	}

	let { file, videoId, onclose }: Props = $props();

	let embedUrl = $derived(`https://www.youtube.com/embed/${videoId}?rel=0`);
	let watchUrl = $derived(`https://www.youtube.com/watch?v=${videoId}`);
</script>

<div class="modal-open modal">
	<div class="modal-box max-w-2xl">
		<button class="btn absolute top-2 right-2 btn-circle btn-ghost btn-sm" onclick={onclose}>
			&times;
		</button>

		<h3 class="text-lg font-bold">YouTube Preview</h3>
		<p class="mt-1 truncate text-sm opacity-60" title={file.name}>{file.name}</p>

		<div class="mt-4">
			<div class="aspect-video w-full overflow-hidden rounded-lg">
				<iframe
					src={embedUrl}
					class="h-full w-full"
					title="YouTube video preview"
					frameborder="0"
					allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
					allowfullscreen
				></iframe>
			</div>

			<div class="mt-3 flex items-center justify-between">
				<span class="badge badge-secondary">{videoId}</span>
				<a href={watchUrl} target="_blank" rel="noopener noreferrer" class="btn btn-ghost btn-sm">
					Open on YouTube
				</a>
			</div>
		</div>
	</div>
	<div class="modal-backdrop" onclick={onclose}></div>
</div>
