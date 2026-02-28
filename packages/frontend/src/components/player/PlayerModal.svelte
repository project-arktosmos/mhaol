<script lang="ts">
	import type { PlayableFile, PlayerConnectionState } from '$types/player.type';
	import PlayerVideo from './PlayerVideo.svelte';

	export let file: PlayableFile;
	export let connectionState: PlayerConnectionState = 'idle';
	export let positionSecs: number = 0;
	export let durationSecs: number | null = null;
	export let onclose: () => void;
</script>

<div class="modal modal-open">
	<div class="modal-box max-w-4xl">
		<button
			class="btn btn-sm btn-circle btn-ghost absolute right-2 top-2"
			onclick={onclose}
		>
			&times;
		</button>

		<h3 class="text-lg font-bold">Now Playing</h3>
		<p class="mt-1 truncate text-sm opacity-60" title={file.name}>{file.name}</p>

		<div class="mt-4">
			<PlayerVideo
				{file}
				{connectionState}
				{positionSecs}
				{durationSecs}
			/>
		</div>
	</div>
	<div class="modal-backdrop" onclick={onclose}></div>
</div>
