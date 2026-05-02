<script lang="ts">
	import { base } from '$app/paths';
	import { CatalogVersionHistoryCard as Shared } from 'cloud-ui';

	interface Props {
		versionHashes: string[];
	}
	let { versionHashes }: Props = $props();

	const hrefFor = (cid: string) => `${base}/catalog/${encodeURIComponent(cid)}`;
</script>

<Shared {versionHashes} {hrefFor} />
