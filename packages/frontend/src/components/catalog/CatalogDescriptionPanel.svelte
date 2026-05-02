<script lang="ts">
	import { base } from '$app/paths';
	import { CatalogDescriptionPanel as Shared, type FirkinReview } from 'cloud-ui';

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
		reviews?: FirkinReview[];
	}

	let {
		description,
		identity = null,
		versionHashes = [],
		reviews = []
	}: Props = $props();

	const hrefFor = (cid: string) => `${base}/catalog/${encodeURIComponent(cid)}`;
</script>

<Shared {description} {identity} {versionHashes} {reviews} {hrefFor} />
