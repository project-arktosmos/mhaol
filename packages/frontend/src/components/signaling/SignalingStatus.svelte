<script lang="ts">
	import classNames from 'classnames';

	export let online: boolean = false;
	export let checking: boolean = false;
</script>

<div class="flex items-center gap-2 text-sm">
	<div
		class={classNames('h-2 w-2 rounded-full', {
			'animate-pulse bg-base-content/40': checking,
			'bg-success': !checking && online,
			'bg-error': !checking && !online
		})}
	></div>
	<span class="opacity-70">
		{#if checking}
			Checking...
		{:else if online}
			Online
		{:else}
			Offline
		{/if}
	</span>
</div>
