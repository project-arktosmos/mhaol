<script lang="ts">
	import classNames from 'classnames';
	import { onMount } from 'svelte';
	import { base } from '$app/paths';
	import { page } from '$app/state';
	import { listSources, type CatalogSource } from '$lib/catalog.service';

	let { classes = 'flex flex-wrap items-center gap-1' }: { classes?: string } = $props();

	let sources = $state<CatalogSource[]>([]);

	onMount(() => {
		void (async () => {
			try {
				sources = await listSources();
			} catch {
				sources = [];
			}
		})();
	});

	const rootPath = `${base}/`;
	const activeAddon = $derived.by(() => {
		if (page.url.pathname !== rootPath) return null;
		const fromUrl = page.url.searchParams.get('addon') ?? '';
		if (fromUrl && sources.some((s) => s.id === fromUrl)) return fromUrl;
		return sources[0]?.id ?? null;
	});
</script>

{#if sources.length > 0}
	<div class={classes}>
		{#each sources as source (source.id)}
			{@const active = activeAddon === source.id}
			<a
				href="{rootPath}?addon={encodeURIComponent(source.id)}"
				data-sveltekit-noscroll
				class={classNames('btn btn-sm', {
					'btn-primary': active,
					'btn-outline': !active
				})}
				title={source.kind}
			>
				{source.label}
			</a>
		{/each}
	</div>
{/if}
