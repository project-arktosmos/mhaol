<script lang="ts">
	import { base } from '$app/paths';
	import { CatalogDescriptionPanel as Shared } from 'cloud-ui';

	interface Identity {
		cid: string;
		createdAt: string;
		updatedAt: string;
		version: number;
	}

	interface Props {
		description: string;
		identity?: Identity | null;
		versionHashes?: string[];
	}

	let { description, identity = null, versionHashes = [] }: Props = $props();

	const hrefFor = (cid: string) => `${base}/catalog/${encodeURIComponent(cid)}`;
</script>

<Shared {description} {identity} {versionHashes} {hrefFor} />
