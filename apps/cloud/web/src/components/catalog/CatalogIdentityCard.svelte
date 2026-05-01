<script lang="ts">
	import { CatalogIdentityCard as Shared } from 'cloud-ui';

	interface Props {
		cid: string;
		createdAt: string;
		updatedAt: string;
		version: number;
	}
	let { cid, createdAt, updatedAt, version }: Props = $props();
</script>

<Shared {cid} {createdAt} {updatedAt} {version} />
