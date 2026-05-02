<script lang="ts">
	import type { Snippet } from 'svelte';
	import { CatalogPageHeader as Shared } from 'cloud-ui';

	interface Props {
		title: string;
		addon: string;
		kindLabel?: string | null;
		year?: number | null;
		extraBadge?: { label: string; class: string } | null;
		actions?: Snippet;
	}

	let { title, addon, kindLabel = null, year = null, extraBadge = null, actions }: Props = $props();
</script>

<Shared {title} {addon} {kindLabel} {year} {extraBadge} {actions} />
