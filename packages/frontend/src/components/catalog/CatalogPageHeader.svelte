<script lang="ts">
	import type { Snippet } from 'svelte';
	import { CatalogPageHeader as Shared } from 'cloud-ui';

	interface Props {
		title: string;
		year?: number | null;
		kindLabel?: string | null;
		actions?: Snippet;
	}

	let { title, year = null, kindLabel = null, actions }: Props = $props();
</script>

<Shared {title} {year} {kindLabel} {actions} />
