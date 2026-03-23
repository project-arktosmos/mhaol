<script lang="ts">
	import classNames from 'classnames';
	import type { Snippet } from 'svelte';

	interface Props {
		connected: boolean;
		connectedLabel?: string;
		disconnectedLabel?: string;
		extra?: Snippet;
		children?: Snippet;
	}

	let {
		connected,
		connectedLabel = 'Server Connected',
		disconnectedLabel = 'Server Disconnected',
		extra,
		children
	}: Props = $props();
</script>

<div
	class={classNames('rounded-lg p-3', {
		'bg-success/10': connected,
		'bg-warning/10': !connected
	})}
>
	<div class="flex items-center justify-between">
		<div class="flex items-center gap-2">
			<div
				class={classNames('h-2 w-2 rounded-full', {
					'bg-success': connected,
					'bg-warning': !connected
				})}
			></div>
			<span class="text-sm font-medium">
				{#if connected}
					{connectedLabel}
				{:else}
					{disconnectedLabel}
				{/if}
			</span>
		</div>
		{#if extra}
			{@render extra()}
		{/if}
	</div>
	{#if children}
		{@render children()}
	{/if}
</div>
