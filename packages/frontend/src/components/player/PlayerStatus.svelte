<script lang="ts">
	import classNames from 'classnames';

	export let serverAvailable: boolean = false;
	export let connectionState: string = 'idle';
</script>

<div class="flex items-center gap-2 text-sm">
	<div
		class={classNames('h-2 w-2 rounded-full', {
			'bg-success': serverAvailable,
			'bg-error': !serverAvailable
		})}
	></div>
	<span class="opacity-70">
		{#if !serverAvailable}
			Stream server offline
		{:else if connectionState === 'streaming'}
			Streaming
		{:else}
			Stream server ready
		{/if}
	</span>
</div>
