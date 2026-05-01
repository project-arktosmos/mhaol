<script lang="ts">
	import type { Snippet } from 'svelte';
	import { base } from '$app/paths';
	import { CatalogPageHeader as Shared } from 'cloud-ui';

	interface Props {
		title: string;
		addon: string;
		kindLabel?: string | null;
		year?: number | null;
		extraBadge?: { label: string; class: string } | null;
		backHref?: string;
		backLabel?: string;
		actions?: Snippet;
	}

	let {
		title,
		addon,
		kindLabel = null,
		year = null,
		extraBadge = null,
		backHref = `${base}/catalog`,
		backLabel = 'Catalog',
		actions
	}: Props = $props();
</script>

<Shared {title} {addon} {kindLabel} {year} {extraBadge} {backHref} {backLabel} {actions} />
