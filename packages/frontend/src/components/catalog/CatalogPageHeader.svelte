<script lang="ts">
	import type { Snippet } from 'svelte';
	import { CatalogPageHeader as Shared } from 'cloud-ui';

	interface Props {
		title: string;
		actions?: Snippet;
	}

	let { title, actions }: Props = $props();
</script>

<Shared {title} {actions} />
