<script lang="ts">
	import type { WebSurfxResult } from 'ui-lib/types/websurfx.type';

	let {
		result
	}: {
		result: WebSurfxResult;
	} = $props();
</script>

<div class="card card-compact bg-base-200 transition-colors hover:bg-base-300">
	<div class="card-body gap-1 p-4">
		<a
			href={result.url}
			target="_blank"
			rel="noopener noreferrer"
			class="link link-primary line-clamp-1 text-sm font-medium"
		>
			{result.title}
		</a>
		<div class="flex items-center gap-2 text-xs text-base-content/50">
			{#if result.domain}
				<span class="badge badge-ghost badge-xs">{result.domain}</span>
			{/if}
			{#if result.publishedDate}
				<span>{result.publishedDate}</span>
			{/if}
			{#if result.provider}
				<span class="badge badge-outline badge-xs">{result.provider}</span>
			{/if}
		</div>
		{#if result.snippet}
			<p class="line-clamp-2 text-xs text-base-content/70">{result.snippet}</p>
		{/if}
		<p class="truncate text-xs text-base-content/40">{result.url}</p>
	</div>
</div>
